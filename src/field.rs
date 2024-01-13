use super::{push_str, ToQuery};

#[derive(Clone)]
pub enum FieldType<'a> {
    Expr(Box<dyn ToQuery + 'a>),
    Field(Vec<Field<'a>>),
    SingleSplat,
    DoubleSplat,
}

#[derive(Clone)]
pub struct Field<'a> {
    /// None if splat
    name: Option<&'a str>,
    fields: FieldType<'a>,
}

impl<'a> Field<'a> {
    pub fn new(name: impl Into<Option<&'a str>>) -> Self {
        Self {
            name: name.into(),
            fields: FieldType::Field(Vec::new()),
        }
    }

    pub fn single_splat(mut self) -> Self {
        self.fields = FieldType::SingleSplat;

        self
    }

    pub fn double_splat(mut self) -> Self {
        self.fields = FieldType::DoubleSplat;

        self
    }

    pub fn expr<T>(mut self, expr: T) -> Self
    where
        T: ToQuery + 'a,
    {
        match &mut self.fields {
            FieldType::Expr(x) => *x = Box::new(expr),
            _ => self.fields = FieldType::Expr(Box::new(expr)),
        }

        self
    }

    pub fn nest(mut self, field: Field<'a>) -> Self {
        match &mut self.fields {
            FieldType::Field(fields) => {
                fields.push(field);
            }
            _ => {
                self.fields = FieldType::Field(vec![field]);
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
    fn to_query_with_indent(&self, indent: usize) -> String {
        let mut qx = String::new();
        let q = &mut qx;

        if let Some(name) = self.name {
            push_str(q, name, indent);
        }

        match &self.fields {
            FieldType::SingleSplat => {
                q.push('*');
                q.push(',');
            }
            FieldType::DoubleSplat => {
                q.push_str("**");
                q.push(',');
            }
            FieldType::Expr(expr) => {
                q.push(' ');
                q.push_str(":=");
                q.push(' ');

                q.push('(');
                q.push('\n');

                q.push_str(&expr.to_query_with_indent(2 + indent));

                q.push('\n');
                push_str(q, ")", indent);
            }
            FieldType::Field(nested_fields) if !nested_fields.is_empty() => {
                q.push(':');
                q.push(' ');

                q.push('{');
                q.push('\n');

                let mut nested_fields = nested_fields.iter();

                if let Some(nested) = nested_fields.next() {
                    q.push_str(&nested.to_query_with_indent(2 + indent));
                }

                for nested in nested_fields {
                    q.push('\n');
                    q.push_str(&nested.to_query_with_indent(2 + indent));
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
        * $(,)?
    ) => {
        [
            Field::new(None).single_splat(),
        ]
    };

    (
        * $(,)?
        $(
            $field:ident $(,)?
            $(: { $($nested_field:ident $(,)?)* } $(,)?)?
            $(: [ $nested_fields:expr $(,)? ] $(,)?)?
            $(:= $expr:expr $(,)?)?
        )+
    ) => {
        [
            Field::new(None).single_splat(),
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
            )+
        ]
    };

    (
        ** $(,)?
    ) => {
        [
            Field::new(None).double_splat(),
        ]
    };

    (
        ** $(,)?
        $(
            $field:ident $(,)?
            $(: { $($nested_field:ident $(,)?)* } $(,)?)?
            $(: [ $nested_fields:expr $(,)? ] $(,)?)?
            $(:= $expr:expr $(,)?)?
        )+
    ) => {
        [
            Field::new(None).double_splat(),
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
            )+
        ]
    };


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

        push_fields(&mut r, fields, 0);

        println!("{r}");
    }

    #[test]
    fn print_single_splat() {
        let another_fields = fields! {
            id,
        };

        fields! { * };
        fields! { *, };

        let fields = fields! {
            *,
            e: [ another_fields ],
            aa := raw("(tag.name, tag.kind)"),
            book_tags: {
                kind,
                name
            },
        };

        let mut r = String::new();

        push_fields(&mut r, fields, 0);

        println!("{r}");
    }

    #[test]
    fn print_double_splat() {
        let another_fields = fields! {
            id,
        };

        fields! { ** };
        fields! { **, };

        let fields = fields! {
            **,
            e: [ another_fields ],
            aa := raw("(tag.name, tag.kind)"),
            book_tags: {
                kind,
                name
            },
        };

        let mut r = String::new();

        push_fields(&mut r, fields, 0);

        println!("{r}");
    }
}
