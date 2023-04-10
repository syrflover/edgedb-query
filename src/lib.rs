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
mod with;

use std::time::SystemTime;

pub use field::*;
pub use filter::*;
pub use forin::*;
pub use group::*;
pub use insert::*;
pub use order_by::*;
pub use select::*;
pub use update::*;
pub use with::*;

use dyn_clone::{clone_trait_object, DynClone};
use edgedb_protocol::{model::LocalDatetime, queryable::Queryable};
use either::Either;
use iter_tools::Itertools;
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

pub const ARG_IDENTITY: &str = "$:";

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

fn push_withs<'a>(q: &mut String, withs: impl IntoIterator<Item = &'a With<'a>>, indent: usize) {
    let mut withs = withs.into_iter();

    if let Some(first) = withs.next() {
        push_str(q, "with", indent);
        q.push('\n');
        q.push_str(&first.to_query_with_indent(2 + indent));
        q.push(',');
        q.push('\n');
    }

    for with in withs {
        q.push_str(&with.to_query_with_indent(2 + indent));
        q.push(',');
        q.push('\n');
    }
}

fn push_fields<'a>(q: &mut String, fields: impl IntoIterator<Item = Field<'a>>, indent: usize) {
    for field in fields {
        q.push_str(&field.to_query_with_indent(indent));
        q.push('\n');
    }
}

fn push_filter<'a>(q: &mut String, filter: Option<&'a Filter<'a>>, indent: usize) {
    if let Some(filter) = filter {
        if filter.is_empty() {
            return;
        }

        push_str(q, "filter", indent);
        q.push('\n');

        q.push_str(&filter.to_query_with_indent(indent));
    }
}

fn push_object<'a>(
    q: &mut String,
    obj: impl IntoIterator<Item = &'a (&'a str, QueryArgOrExpr<'a>)>,
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
                q.push_str(&value.to_query_arg());
            }

            Either::Right(expr) => {
                q.push('(');
                q.push('\n');

                q.push_str(&expr.to_query_with_indent(4 + indent));

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

type QueryArgOrExpr<'a> = Either<Box<dyn ToQueryArg + 'a>, Box<dyn ToQuery + 'a>>;

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
    fn to_query_with_indent(&self, indent: usize) -> String {
        let mut qx = String::new();
        let q = &mut qx;

        push_str(q, self.0, indent);

        qx
    }
}

impl<'a> ToQueryArg for Raw<'a> {
    fn to_query_arg(&self) -> String {
        self.0.to_string()
    }
}

pub trait ToQuery: DynClone + Send + Sync {
    fn to_query_with_indent(&self, indent: usize) -> String;

    fn to_query(&self) -> String {
        self.to_query_with_indent(0)
            .tap(|query| tracing::debug!("\n{query}"))
    }
}

clone_trait_object!(ToQuery);

impl<T: Clone + ToQuery> ToQuery for &T {
    fn to_query_with_indent(&self, indent: usize) -> String {
        (*self).to_query_with_indent(indent)
    }
}

#[async_trait::async_trait]
pub trait QueryExecution: Sized {
    async fn query<T: Queryable + Send>(
        self,
        edgedb: &edgedb_tokio::Client,
    ) -> Result<Vec<T>, edgedb_tokio::Error>;

    async fn query_single<T: Queryable + Send>(
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
    async fn query<T: Queryable + Send>(
        self,
        edgedb: &edgedb_tokio::Client,
    ) -> Result<Vec<T>, edgedb_tokio::Error> {
        elapsed!(edgedb.query::<T, _>(&self.to_query(), &()).await)
    }

    async fn query_single<T: Queryable + Send>(
        self,
        edgedb: &edgedb_tokio::Client,
    ) -> Result<Option<T>, edgedb_tokio::Error> {
        elapsed!(edgedb.query_single::<T, _>(&self.to_query(), &()).await)
    }

    async fn query_json(
        self,
        edgedb: &edgedb_tokio::Client,
    ) -> Result<edgedb_protocol::model::Json, edgedb_tokio::Error> {
        elapsed!(edgedb.query_json(&self.to_query(), &()).await)
    }

    async fn query_single_json(
        self,
        edgedb: &edgedb_tokio::Client,
    ) -> Result<Option<edgedb_protocol::model::Json>, edgedb_tokio::Error> {
        elapsed!(edgedb.query_single_json(&self.to_query(), &()).await)
    }
}

pub trait ToQueryArg: DynClone + Send + Sync {
    fn to_query_arg(&self) -> String;
}

clone_trait_object!(ToQueryArg);

impl<T> ToQueryArg for Vec<T>
where
    T: ToQueryArg + Clone,
{
    fn to_query_arg(&self) -> String {
        (&self).to_query_arg()
    }
}

impl<T> ToQueryArg for &Vec<T>
where
    T: ToQueryArg,
{
    fn to_query_arg(&self) -> String {
        let r = self.iter().map(|x| x.to_query_arg()).join(", ");

        format!("{{ {r} }}")
    }
}

impl ToQueryArg for String {
    fn to_query_arg(&self) -> String {
        self.as_str().to_query_arg()
    }
}

impl ToQueryArg for &String {
    fn to_query_arg(&self) -> String {
        self.as_str().to_query_arg()
    }
}

impl ToQueryArg for &str {
    fn to_query_arg(&self) -> String {
        let escaped_single_quote = self.replace('\'', "\\'");
        format!("<str>'{escaped_single_quote}'")
    }
}

impl ToQueryArg for edgedb_protocol::model::Datetime {
    fn to_query_arg(&self) -> String {
        let datetime = LocalDatetime::from(*self);
        format!("<datetime>'{}T{}+00'", datetime.date(), datetime.time())
    }
}

macro_rules! impl_to_query_arg {
    ($($ty:ty $(,)?)*) => {
        $(
            impl ToQueryArg for $ty {
                fn to_query_arg(&self) -> String {
                    // format!("<{}>{self}", stringify!($ty))
                    self.to_string()
                }
            }
        )*
    };
}

// fn cast_type(x: &str) -> Option<&str> {
//     match x {
//         "bool" => "bool",
//         "f32" => "f32",
//         "f64" => "f64",
//         "Uuid" => "uuid"
//     }
// }

impl_to_query_arg![i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, bool];

macro_rules! impl_to_query_arg_for_tuple {
    ($($name:ident $(,)?)+) => {

        impl<$($name,)+> ToQueryArg for ($($name,)+)
        where
            $(
                $name: ToQueryArg + Clone,
            )+
        {
            fn to_query_arg(&self) -> String {
                #[allow(non_snake_case)]
                let ($($name,)+) = self;
                let mut q = String::new();

                q.push('(');

                $(
                    q.push_str(&$name.to_query_arg());
                    q.push(',');
                )+

                q.push(')');

                q

            }
        }
};
}

impl_to_query_arg_for_tuple![T1];
impl_to_query_arg_for_tuple![T1, T2];
impl_to_query_arg_for_tuple![T1, T2, T3];
impl_to_query_arg_for_tuple![T1, T2, T3, T4];
impl_to_query_arg_for_tuple![T1, T2, T3, T4, T5];
impl_to_query_arg_for_tuple![T1, T2, T3, T4, T5, T6];
impl_to_query_arg_for_tuple![T1, T2, T3, T4, T5, T6, T7];
impl_to_query_arg_for_tuple![T1, T2, T3, T4, T5, T6, T7, T8];
impl_to_query_arg_for_tuple![T1, T2, T3, T4, T5, T6, T7, T8, T9];
impl_to_query_arg_for_tuple![T1, T2, T3, T4, T5, T6, T7, T8, T9, T10];
impl_to_query_arg_for_tuple![T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11];
impl_to_query_arg_for_tuple![T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12];

// TODO:
// - f32, f64
// - type cast <type>'aa'
