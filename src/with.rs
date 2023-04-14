use either::Either;

use super::*;

#[derive(Clone)]
pub struct With<'a> {
    pub name: &'a str,
    x: Option<Either<Value<'a>, Box<dyn ToQuery + 'a>>>,
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

    pub fn value(mut self, value: impl IntoValue<'a>) -> Self {
        self.x.replace(Either::Left(value.into_value()));

        self
    }
}

pub fn with<'a>(name: &'a str, value: impl IntoValue<'a>) -> With<'a> {
    With::new(name).value(value)
}

pub fn with_expr<'a, T>(name: &'a str, expr: T) -> With<'a>
where
    T: ToQuery + 'a,
{
    With::new(name).expr(expr)
}

impl<'a> ToQuery for With<'a> {
    fn to_query_with_indent(&mut self, ctx: &mut Context, indent: usize) -> String {
        let mut qx = String::new();
        let q = &mut qx;

        push_str(q, self.name, indent);
        q.push_str(" := ");

        match self.x.take().expect("not set value from With") {
            Either::Left(value) => {
                push_arg(q, ctx, value);
            }
            Either::Right(mut expr) => {
                q.push('(');
                q.push('\n');

                let query = expr.to_query_with_indent(ctx, 2 + indent);

                q.push_str(&query);

                q.push('\n');
                push_str(q, ")", indent);
            }
        }

        qx
    }
}
