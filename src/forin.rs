use super::*;

#[derive(Clone)]
pub struct ForInBuilder<'a> {
    pub arr: &'a str,
    pub elem: &'a str,
    withs: Vec<With<'a>>,
    expr: Option<Box<dyn ToQuery + 'a>>,
}

pub fn forin<'a>(arr_name: &'a str, elem_name: &'a str) -> ForInBuilder<'a> {
    ForInBuilder::new(arr_name, elem_name)
}

impl<'a> ForInBuilder<'a> {
    pub fn new(arr_name: &'a str, elem_name: &'a str) -> Self {
        Self {
            arr: arr_name,
            elem: elem_name,
            expr: None,
            withs: Vec::new(),
        }
    }

    pub fn with(mut self, with: With<'a>) -> Self {
        self.withs.push(with);

        self
    }

    pub fn expr<T>(mut self, expr: T) -> Self
    where
        T: ToQuery + 'a,
    {
        self.expr.replace(Box::new(expr));

        self
    }
}

impl<'a> ToQuery for ForInBuilder<'a> {
    fn to_query_with_indent(&mut self, ctx: &mut Context, indent: usize) -> String {
        let mut qx = String::new();
        let q = &mut qx;

        {
            if !self.withs.is_empty() {
                push_withs(q, ctx, std::mem::take(&mut self.withs), indent);
            }
        }

        push_str(q, "for ", indent);
        q.push_str(self.elem);
        q.push_str(" in ");
        q.push_str(self.arr);
        q.push_str(" union ");

        q.push('(');
        q.push('\n');

        let mut expr = self.expr.take().expect("not set `expr` from ForInBuilder");

        let query = expr.to_query_with_indent(ctx, 2 + indent);

        q.push_str(&query);

        q.push('\n');

        push_str(q, ")", indent);

        qx
    }
}
