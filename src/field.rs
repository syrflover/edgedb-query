use either::Either;

use super::*;

#[derive(Clone)]
pub struct Field<'a> {
    name: &'a str,
    fields: Either<Box<dyn ToQuery + 'a>, Vec<Field<'a>>>,
}

impl<'a> Field<'a> {
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            fields: Either::Right(Vec::new()),
        }
    }

    pub fn expr<T>(mut self, expr: T) -> Self
    where
        T: ToQuery + 'a,
    {
        match &mut self.fields {
            Either::Left(x) => *x = Box::new(expr),
            Either::Right(_fields) => self.fields = Either::Left(Box::new(expr)),
        }

        self
    }

    pub fn nest(mut self, field: Field<'a>) -> Self {
        match &mut self.fields {
            Either::Left(_expr) => {
                self.fields = Either::Right(vec![field]);
            }
            Either::Right(fields) => {
                fields.push(field);
            }
        }

        self
    }

    pub fn nests<T>(mut self, fields: T) -> Self
    where
        T: IntoIterator<Item = Field<'a>> + 'a + Clone,
    {
        for field in fields {
            self = self.nest(field);
        }

        self
    }
}

impl<'a> ToQuery for Field<'a> {
    fn to_query_with_indent(&mut self, ctx: &mut Context, indent: usize) -> String {
        let mut qx = String::new();
        let q = &mut qx;

        push_str(q, self.name, indent);

        match self.fields.as_mut() {
            Either::Left(expr) => {
                q.push(' ');
                q.push_str(":=");
                q.push(' ');

                q.push('(');
                q.push('\n');

                let query = expr.to_query_with_indent(ctx, 2 + indent);

                q.push_str(&query);

                q.push('\n');
                push_str(q, ")", indent);
            }
            Either::Right(nested_fields) if !nested_fields.is_empty() => {
                q.push(':');
                q.push(' ');

                q.push('{');
                q.push('\n');

                let mut nested_fields = nested_fields.into_iter();

                if let Some(nested) = nested_fields.next() {
                    let query = nested.to_query_with_indent(ctx, 2 + indent);

                    q.push_str(&query);
                }

                for nested in nested_fields {
                    q.push('\n');

                    let query = nested.to_query_with_indent(ctx, 2 + indent);

                    q.push_str(&query);
                }

                q.push('\n');
                push_str(q, "}", indent);
                q.push(',');
            }
            _ => {
                q.push(',');
            }
        }

        qx
    }
}

/// ```ignore
///
/// let another_fields = fields! { a, b, c };
///
/// fields! {
///     id,
///     title,
///     tag := raw("(tag.kind, tag.name)"),
///     book_tags: {
///         kind,
///         name,
///     },
///     aaaa: [ another_fields ]
/// }
/// ```
#[macro_export]
macro_rules! fields {
    (
        $(
            $field:ident $(,)?
            $(: { $($nested_field:ident $(,)?)* } $(,)?)?
            $(: [ $nested_fields:expr $(,)? ] $(,)?)?
            $(:= $expr:expr $(,)?)?
        )*
    ) => {
        [
            $(
                Field::new(stringify!($field))
                $(
                    $(
                        .nest(Field::new(stringify!($nested_field)))
                    )*
                )?
                $(
                    .nests($nested_fields)
                )?
                $(
                    .expr($expr)
                )?
                ,
            )*
        ]
    };

}

// fn a() {
//     let r = fields! {
//         title,
//         book_tags: {
//             kind,
//             name
//         },
//     };
// }

#[cfg(test)]
mod tests {
    use crate::{push_fields, raw};

    use super::*;

    #[test]
    fn print() {
        let another_fields = fields! {
            id,
        };

        let fields = fields! {
            title,
            e: [ another_fields ],
            aa := raw("(tag.name, tag.kind)"),
            book_tags: {
                kind,
                name
            },
        };

        let mut r = String::new();

        push_fields(&mut r, &mut Context::new(), fields, 0);

        println!("{r}");
    }
}
