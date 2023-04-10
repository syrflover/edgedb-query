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
    fn to_query_with_indent(&self, indent: usize) -> String {
        let fields = self.fields.clone().into_iter();

        let mut qx = String::new();
        let q = &mut qx;

        // with
        {
            push_withs(q, &self.withs, indent);
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

                push_fields(q, self.fields.clone(), 2 + indent);

                push_str(q, "}", indent);
            }
        }

        // expr
        {
            if let Some(expr) = &self.expr {
                q.push(' ');
                // q.push('(');
                q.push('\n');

                q.push_str(&expr.to_query_with_indent(2 + indent));

                q.push('\n');
                push(q, ')', indent);
            }
        }

        // filter
        {
            if self.filter.is_some() {
                q.push('\n');
            }

            push_filter(q, self.filter.as_ref(), indent);
        }

        // order by
        {
            let mut orders = self.orders.iter();

            if let Some(first) = orders.next() {
                q.push('\n');
                push_str(q, "order by", indent);
                q.push('\n');
                q.push_str(&first.to_query_with_indent(indent));
            }

            for ord in orders {
                q.push(' ');
                q.push_str("then");
                q.push('\n');

                q.push_str(&ord.to_query_with_indent(indent));
            }
        }

        // offset
        if let Some(skip) = self.skip {
            if skip > 0 {
                q.push('\n');
                push_str(q, "offset ", indent);
                q.push_str(&skip.to_string());
            }
        }

        // limit
        if let Some(take) = self.take {
            q.push('\n');
            push_str(q, "limit ", indent);
            q.push_str(&take.to_string());
        }

        qx
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::repository::edgedb::query::order_by::{order_by, ASC, DESC};

//     use super::*;
//     use FilterOperator::*;

//     #[test]
//     fn print() {
//         let mut r = SelectBuilder::new("Book", "uid, title, book_tags: { kind, name }");

//         r.filter(
//             None,
//             Filter::new().add(None, ".uid = $:", 2217180).add(
//                 And,
//                 ".kind = <BookKind>'$:'",
//                 "Female",
//             ),
//         );

//         let r = r.to_query();

//         println!("{r}");

//         println!("---------------------");

//         let mut r = SelectBuilder::new("Book", "uid, title, book_tags: { kind, name }");

//         r.filter(
//             None,
//             Filter::new().add(None, ".uid = $:", 2217180).add(
//                 And,
//                 ".kind = <BookKind>'$:'",
//                 "Female",
//             ),
//         );

//         r.filter(None, Filter::new().add(None, ".title = $:", "'sex'"));

//         r.order_by(order_by(".uid", DESC))
//             .order_by(order_by(".title", ASC));

//         r.skip(25);

//         r.take(100);

//         // r.with(With::new("abcd", "select book"));

//         let r = r.to_query();

//         println!("{r}");
//     }
// }
