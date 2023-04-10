use either::Either;

use super::{push_str, ToQuery, ToQueryArg};

#[derive(Clone)]
pub struct With<'a> {
    pub name: &'a str,
    x: Option<Either<Box<dyn ToQueryArg + 'a>, Box<dyn ToQuery + 'a>>>,
}

impl<'a> With<'a> {
    pub fn new(name: &'a str) -> Self {
        Self { name, x: None }
    }

    pub fn expr<T>(mut self, expr: T) -> Self
    where
        T: ToQuery + 'a,
    {
        self.x.replace(Either::Right(Box::new(expr)));

        self
    }

    pub fn value<T>(mut self, value: T) -> Self
    where
        T: ToQueryArg + 'a,
    {
        self.x.replace(Either::Left(Box::new(value)));

        self
    }
}

pub fn with<'a, T>(name: &'a str, value: T) -> With<'a>
where
    T: ToQueryArg + 'a,
{
    With::new(name).value(value)
}

pub fn with_expr<'a, T>(name: &'a str, expr: T) -> With<'a>
where
    T: ToQuery + 'a,
{
    With::new(name).expr(expr)
}

impl<'a> ToQuery for With<'a> {
    fn to_query_with_indent(&self, indent: usize) -> String {
        let mut qx = String::new();
        let q = &mut qx;

        push_str(q, self.name, indent);
        q.push_str(" := ");

        match self.x.as_ref().expect("not set value from With") {
            Either::Left(value) => {
                q.push_str(&value.to_query_arg());
            }
            Either::Right(expr) => {
                q.push('(');
                q.push('\n');

                q.push_str(&expr.to_query_with_indent(2 + indent));

                q.push('\n');
                push_str(q, ")", indent);
            }
        }

        qx
    }
}
