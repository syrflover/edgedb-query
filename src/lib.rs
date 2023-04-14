//! QueryBuilder 만들 때 주의 점
//! - 소괄호로 해당 쿼리를 감쌀 때는 해당 쿼리 빌더 안에서 해야함. 바깥 빌더에서 소괄호를 감싸면 indent에 문제 생김
mod field;
mod filter;
mod forin;
mod group;
mod insert;
mod order_by;
mod select;
mod update;
mod value;
mod with;

pub use field::*;
pub use filter::*;
pub use forin::*;
pub use group::*;
pub use insert::*;
pub use order_by::*;
pub use select::*;
#[cfg(test)]
pub use tests::*;
pub use update::*;
pub use value::*;
pub use with::*;

use std::time::SystemTime;

use dyn_clone::{clone_trait_object, DynClone};
use edgedb_protocol::QueryResult;
use either::Either;
use tap::Tap;

/// ### Example
/// ```
/// queryable!(User, [(name, String), (email, String)]);
/// ```
#[macro_export]
macro_rules! queryable {
    ($ident:ident, [$(($prop:ident, $prop_ty:ty) $(,)?)*]) => {
        #[allow(dead_code)]
        #[derive(Debug, ::edgedb_derive::Queryable)]
        struct $ident {
            $(
                $prop: $prop_ty,
            )*
        }
    };
}

pub const ARG_IDENTITY: &str = "$?";

fn push(q: &mut String, char: char, indent: usize) {
    for _ in 0..indent {
        q.push(' ');
    }

    q.push(char);
}

fn push_str(q: &mut String, string: &str, indent: usize) {
    for _ in 0..indent {
        q.push(' ');
    }

    q.push_str(string);
}

fn replace_arg<'a>(x: &str, ctx: &'a mut Context, arg: impl IntoValue<'a>) -> String {
    let arg = arg.into_value();

    let r = x.replacen(
        ARG_IDENTITY,
        &format!("<{}>${}", arg.kind, ctx.args.len()),
        1,
    );

    ctx.args.push(arg.inner);

    r
}

fn push_arg<'a>(q: &mut String, ctx: &'a mut Context, arg: impl IntoValue<'a>) {
    let arg = arg.into_value();

    q.push('<');
    q.push_str(&arg.kind);
    q.push('>');
    q.push('$');
    q.push_str(&ctx.args.len().to_string());

    ctx.args.push(arg.inner);
}

fn push_withs<'a>(
    q: &mut String,
    ctx: &'a mut Context,
    withs: impl IntoIterator<Item = With<'a>>,
    indent: usize,
) {
    let mut withs = withs.into_iter();

    if let Some(mut first) = withs.next() {
        push_str(q, "with", indent);
        q.push('\n');

        let query = first.to_query_with_indent(ctx, 2 + indent);

        q.push_str(&query);
        q.push(',');
        q.push('\n');
    }

    for mut with in withs {
        let query = with.to_query_with_indent(ctx, 2 + indent);

        q.push_str(&query);
        q.push(',');
        q.push('\n');
    }
}

fn push_fields<'a>(
    q: &mut String,
    ctx: &mut Context,
    fields: impl IntoIterator<Item = Field<'a>>,
    indent: usize,
) {
    for mut field in fields {
        let query = field.to_query_with_indent(ctx, indent);

        q.push_str(&query);
        q.push('\n');
    }
}

fn push_filter<'a>(
    q: &mut String,
    ctx: &'a mut Context,
    filter: Option<Filter<'a>>,
    indent: usize,
) {
    if let Some(mut filter) = filter {
        if filter.is_empty() {
            return;
        }

        push_str(q, "filter", indent);
        q.push('\n');

        let query = filter.to_query_with_indent(ctx, indent);

        q.push_str(&query);
    }
}

fn push_object<'a>(
    q: &mut String,
    ctx: &'a mut Context,
    obj: impl IntoIterator<Item = (&'a str, QueryArgOrExpr<'a>)>,
    indent: usize,
) {
    q.push('{');

    for (field, value) in obj {
        q.push('\n');
        push_str(q, field, 2 + indent);

        q.push(' ');
        q.push_str(":=");
        q.push(' ');

        match value {
            Either::Left(value) => {
                push_arg(q, ctx, value);
                // q.push_str(&value.to_query_arg());
            }

            Either::Right(mut expr) => {
                q.push('(');
                q.push('\n');

                let query = expr.to_query_with_indent(ctx, 4 + indent);

                q.push_str(&query);

                q.push('\n');
                push(q, ')', 2 + indent);
            }
        }

        q.push(',');
    }

    q.push('\n');
    push(q, '}', indent);
}

/// tag.<book_tags[is Book]
pub fn backlink(target: &str, property: &str, is: &str) -> String {
    format!("{target}.<{property}[is {is}]")
}

pub trait TypeName {
    fn type_name() -> &'static str;
}

type QueryArgOrExpr<'a> = Either<Value<'a>, Box<dyn ToQuery + 'a>>;

#[derive(Clone)]
pub struct Raw<'a>(&'a str);

pub fn raw(raw: &str) -> Raw {
    Raw::new(raw)
}

impl<'a> Raw<'a> {
    pub fn new(raw: &'a str) -> Self {
        Self(raw)
    }
}

impl<'a> ToQuery for Raw<'a> {
    fn to_query_with_indent(&mut self, _ctx: &mut Context, indent: usize) -> String {
        let mut qx = String::new();
        let q = &mut qx;

        push_str(q, self.0, indent);

        qx
    }
}

#[derive(Default)]
pub struct Context {
    pub(crate) args: Vec<edgedb_protocol::value::Value>,
}

impl Context {
    pub fn new() -> Self {
        Self { args: Vec::new() }
    }
}

pub trait ToQuery: DynClone + Send + Sync {
    fn to_query_with_indent(&mut self, ctx: &mut Context, indent: usize) -> String;

    fn to_query(&mut self) -> (String, Context) {
        let mut ctx = Context::new();

        let query = self.to_query_with_indent(&mut ctx, 0);

        tracing::debug!("\n{query}");
        tracing::debug!("query args: {:?}", ctx.args);

        (query, ctx)
    }
}

clone_trait_object!(ToQuery);

#[async_trait::async_trait]
pub trait QueryExecution: Sized {
    async fn query<T: QueryResult + Send>(
        self,
        edgedb: &edgedb_tokio::Client,
    ) -> Result<Vec<T>, edgedb_tokio::Error>;

    async fn query_single<T: QueryResult + Send>(
        self,
        edgedb: &edgedb_tokio::Client,
    ) -> Result<Option<T>, edgedb_tokio::Error>;

    async fn query_json(
        self,
        edgedb: &edgedb_tokio::Client,
    ) -> Result<edgedb_protocol::model::Json, edgedb_tokio::Error>;

    async fn query_single_json(
        self,
        edgedb: &edgedb_tokio::Client,
    ) -> Result<Option<edgedb_protocol::model::Json>, edgedb_tokio::Error>;
}

/// for query exectuion
macro_rules! elapsed {
    ($expr:expr) => {{
        let timer = SystemTime::now();
        $expr.tap(|_| {
            let elapsed = timer.elapsed().unwrap_or_default().as_millis();
            tracing::info!("query execution time: {} ms", elapsed)
        })
    }};
}

#[async_trait::async_trait]
impl<Q> QueryExecution for Q
where
    Q: ToQuery,
{
    async fn query<T: QueryResult + Send>(
        mut self,
        edgedb: &edgedb_tokio::Client,
    ) -> Result<Vec<T>, edgedb_tokio::Error> {
        let (query, ctx) = self.to_query();

        elapsed!(edgedb.query::<T, _>(&query, &ctx.args).await)
    }

    async fn query_single<T: QueryResult + Send>(
        mut self,
        edgedb: &edgedb_tokio::Client,
    ) -> Result<Option<T>, edgedb_tokio::Error> {
        let (query, ctx) = self.to_query();

        // let shape = (0..ctx.args.len())
        //     .map(|i| descriptors::TupleElement {
        //         name: i.to_string(),
        //         type_pos: descriptors::TypePos(0),
        //     })
        //     .collect::<Vec<_>>();

        // let args = Value::NamedTuple {
        //     shape: NamedTupleShape::from(&*shape),
        //     fields: ctx.args,
        // };

        // let args = Value::SparseObject(SparseObject::from_pairs(
        //     ctx.args
        //         .into_iter()
        //         .enumerate()
        //         .map(|(i, arg)| (i.to_string(), arg)),
        // ));

        // let args = Value::Tuple(ctx.args);

        elapsed!(edgedb.query_single::<T, _>(&query, &ctx.args).await)
    }

    async fn query_json(
        mut self,
        edgedb: &edgedb_tokio::Client,
    ) -> Result<edgedb_protocol::model::Json, edgedb_tokio::Error> {
        let (query, ctx) = self.to_query();

        elapsed!(edgedb.query_json(&query, &ctx.args).await)
    }

    async fn query_single_json(
        mut self,
        edgedb: &edgedb_tokio::Client,
    ) -> Result<Option<edgedb_protocol::model::Json>, edgedb_tokio::Error> {
        let (query, ctx) = self.to_query();

        elapsed!(edgedb.query_single_json(&query, &ctx.args).await)
    }
}

#[cfg(test)]
mod tests {
    use edgedb_tokio::TlsSecurity;

    fn tracing() {
        if std::env::args().any(|arg| arg == "--nocapture") {
            let subscriber = tracing_subscriber::fmt()
                .with_max_level(tracing::Level::DEBUG)
                .with_line_number(true)
                .finish();

            tracing::subscriber::set_global_default(subscriber).ok();
        }
    }

    pub async fn setup() -> Result<edgedb_tokio::Client, edgedb_tokio::Error> {
        tracing();
        dotenv::dotenv().ok();

        let edgedb_dsn = std::env::var("EDGEDB_DSN").unwrap();

        let config = edgedb_tokio::Builder::new()
            .dsn(&edgedb_dsn)?
            .tls_security(TlsSecurity::Insecure)
            .build_env()
            .await?;

        let client = edgedb_tokio::Client::new(&config);

        client.ensure_connected().await?;

        Ok(client)
    }
}
