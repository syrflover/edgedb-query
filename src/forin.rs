use super::{push_str, push_withs, ToQuery, With};

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
    fn to_query_with_indent(&self, indent: usize) -> String {
        let mut qx = String::new();
        let q = &mut qx;

        {
            if !self.withs.is_empty() {
                push_withs(q, self.withs.iter(), indent);
            }
        }

        push_str(q, "for ", indent);
        q.push_str(self.elem);
        q.push_str(" in ");
        q.push_str(self.arr);
        q.push_str(" union ");

        q.push('(');
        q.push('\n');

        let expr = self
            .expr
            .as_ref()
            .expect("not set `expr` from ForInBuilder");

        q.push_str(&expr.to_query_with_indent(2 + indent));

        q.push('\n');

        push_str(q, ")", indent);

        qx
    }
}
