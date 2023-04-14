use std::borrow::Cow;

use super::*;

#[derive(Clone)]
pub struct SelectBuilder<'a, T: 'a> {
    target: Cow<'a, str>,
    fields: T,
    withs: Vec<With<'a>>,
    filter: Option<Filter<'a>>,
    skip: Option<usize>,
    take: Option<usize>,
    orders: Vec<OrderBy<'a>>,
    distinct: bool,
    expr: Option<Box<dyn ToQuery + 'a>>,
}

pub fn select<'a, T>(target: impl Into<Cow<'a, str>>, fields: T) -> SelectBuilder<'a, T>
where
    T: IntoIterator<Item = Field<'a>>,
{
    SelectBuilder::new(target, fields)
}

pub fn select_expr<'a, T>(expr: T) -> SelectBuilder<'a, [Field<'a>; 0]>
where
    T: ToQuery + 'a,
{
    SelectBuilder::new("", []).expr(expr)
}

impl<'a, T> SelectBuilder<'a, T>
where
    T: IntoIterator<Item = Field<'a>>,
{
    pub fn new(target: impl Into<Cow<'a, str>>, fields: T) -> Self {
        Self {
            target: target.into(),
            fields,
            withs: Vec::new(),
            filter: None,
            skip: None,
            take: None,
            orders: Vec::new(),
            distinct: false,
            expr: None,
        }
    }

    pub fn with(mut self, with: With<'a>) -> Self {
        self.withs.push(with);

        self
    }

    pub fn distinct(mut self, distinct: bool) -> Self {
        self.distinct = distinct;

        self
    }

    pub fn expr<U>(mut self, expr: U) -> Self
    where
        U: ToQuery + 'a,
    {
        self.expr.replace(Box::new(expr));

        self
    }

    pub fn filter(mut self, filter: Filter<'a>) -> Self {
        self.filter.replace(filter);

        self
    }

    pub fn skip(mut self, n: usize) -> Self {
        self.skip.replace(n);

        self
    }

    pub fn take(mut self, n: usize) -> Self {
        self.take.replace(n);

        self
    }

    pub fn order_by(mut self, ord: OrderBy<'a>) -> Self {
        self.orders.push(ord);

        self
    }
}

impl<'a, T> ToQuery for SelectBuilder<'a, T>
where
    T: IntoIterator<Item = Field<'a>> + Send + Sync + Clone + 'a,
{
    fn to_query_with_indent(&mut self, ctx: &mut Context, indent: usize) -> String {
        let fields = self.fields.clone().into_iter();

        let mut qx = String::new();
        let q = &mut qx;

        // with
        {
            push_withs(q, ctx, std::mem::take(&mut self.withs), indent);
        }

        // select
        {
            push_str(q, "select", indent);
            q.push(' ');

            if self.distinct {
                q.push_str("distinct");
                q.push(' ');
            }

            q.push_str(self.target.as_ref());

            if fields.peekable().count() > 0 {
                q.push(' ');
                q.push('{');
                q.push('\n');

                push_fields(q, ctx, self.fields.clone(), 2 + indent);

                push_str(q, "}", indent);
            }
        }

        // expr
        {
            if let Some(mut expr) = self.expr.take() {
                q.push(' ');
                // q.push('(');
                q.push('\n');

                let query = expr.to_query_with_indent(ctx, 2 + indent);

                q.push_str(&query);

                q.push('\n');
                push(q, ')', indent);
            }
        }

        // filter
        {
            if self.filter.is_some() {
                q.push('\n');
            }

            push_filter(q, ctx, self.filter.take(), indent);
        }

        // order by
        {
            let mut orders = std::mem::take(&mut self.orders).into_iter();

            if let Some(mut first) = orders.next() {
                q.push('\n');
                push_str(q, "order by", indent);
                q.push('\n');

                let query = first.to_query_with_indent(ctx, indent);

                q.push_str(&query);
            }

            for mut ord in orders {
                q.push(' ');
                q.push_str("then");
                q.push('\n');

                let query = ord.to_query_with_indent(ctx, indent);

                q.push_str(&query);
            }
        }

        // offset
        if let Some(skip) = self.skip {
            if skip > 0 {
                q.push('\n');
                push_str(q, "offset ", indent);
                push_arg(q, ctx, skip as i32)
            }
        }

        // limit
        if let Some(take) = self.take {
            q.push('\n');
            push_str(q, "limit ", indent);
            push_arg(q, ctx, take as i32);
        }

        qx
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print() {
        let (r, _) = select("target", fields! { id })
            .filter(filter().add(AND, "a = $?", 1))
            .to_query();

        println!("{r}");
    }

    #[tokio::test]
    async fn select_args() {
        let client = setup().await.unwrap();

        // let r = client
        //     .query::<i32, _>(
        //         "select <array<int32>>$0",
        //         &Value::Array(vec![Value::Int32(12345)]),
        //     )
        //     .await
        //     .unwrap();

        let r = select("Book", [])
            .with(with("args", 12345))
            .query_single::<i32>(&client)
            .await
            .unwrap();

        tracing::debug!("{r:?}");

        // assert_eq!(r, Some((12345,)));
    }
}
