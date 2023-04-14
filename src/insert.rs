use either::Either;

use super::*;

#[derive(Clone)]
pub struct OnConflict<'a> {
    field: Option<&'a str>,
    else_expr: Option<Box<dyn ToQuery + 'a>>,
}

pub fn on_conflict<'a>() -> OnConflict<'a> {
    OnConflict::new()
}

pub fn do_nothing<'a>() -> OnConflict<'a> {
    OnConflict::new().do_nothing()
}

impl<'a> OnConflict<'a> {
    pub fn new() -> Self {
        Self {
            field: None,
            else_expr: None,
        }
    }

    pub fn do_nothing(mut self) -> Self {
        self.else_expr.take();

        self
    }

    pub fn field(mut self, field: &'a str) -> Self {
        self.field.replace(field);

        self
    }

    pub fn else_expr<T>(mut self, expr: T) -> Self
    where
        T: ToQuery + 'a,
    {
        self.else_expr.replace(Box::new(expr));

        self
    }
}

impl<'a> Default for OnConflict<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> ToQuery for OnConflict<'a> {
    fn to_query_with_indent(&mut self, ctx: &mut Context, indent: usize) -> String {
        let mut qx = String::new();
        let q = &mut qx;

        push_str(q, "unless conflict", indent);

        if let (Some(field), Some(mut expr)) = (self.field, self.else_expr.take()) {
            q.push(' ');
            q.push_str("on");
            q.push(' ');

            q.push_str(field);

            q.push('\n');
            push_str(q, "else", indent);
            q.push(' ');
            q.push('(');
            q.push('\n');

            let query = expr.to_query_with_indent(ctx, 2 + indent);

            q.push_str(&query);
            q.push('\n');
            push(q, ')', indent);
        }

        qx
    }
}

#[derive(Clone)]
pub struct InsertBuilder<'a> {
    target: &'a str,
    withs: Vec<With<'a>>,
    values: Vec<(&'a str, QueryArgOrExpr<'a>)>,
    on_conflict: Option<OnConflict<'a>>,
}

pub fn insert(target: &str) -> InsertBuilder {
    InsertBuilder::new(target)
}

impl<'a> InsertBuilder<'a> {
    pub fn new(target: &'a str) -> Self {
        Self {
            target,
            withs: Vec::new(),
            values: Vec::new(),
            on_conflict: None,
        }
    }

    pub fn with(mut self, with: With<'a>) -> Self {
        self.withs.push(with);

        self
    }

    pub fn set(mut self, field: &'a str, v: impl IntoValue<'a>) -> Self {
        self.values.push((field, Either::Left(v.into_value())));

        self
    }

    pub fn set_expr<T>(mut self, field: &'a str, v: T) -> Self
    where
        T: ToQuery + 'a,
    {
        self.values.push((field, Either::Right(Box::new(v))));

        self
    }

    pub fn on_conflict(mut self, on_conflict: OnConflict<'a>) -> Self {
        self.on_conflict.replace(on_conflict);

        self
    }
}

impl<'a> ToQuery for InsertBuilder<'a> {
    fn to_query_with_indent(&mut self, ctx: &mut Context, indent: usize) -> String {
        let mut qx = String::new();
        let q = &mut qx;

        // with
        {
            push_withs(q, ctx, std::mem::take(&mut self.withs), indent);
        }

        push_str(q, "insert", indent);
        q.push(' ');
        q.push_str(self.target);
        q.push(' ');

        // set values
        {
            push_object(q, ctx, std::mem::take(&mut self.values), indent);
        }

        // on conflict
        if let Some(mut on_conflict) = self.on_conflict.take() {
            q.push('\n');

            let query = on_conflict.to_query_with_indent(ctx, indent);

            q.push_str(&query);
        }

        qx
    }
}

#[cfg(test)]
mod tests {
    // use crate::repository::edgedb::prelude::{filter, select};

    // use super::*;

    // #[test]
    // fn print() {
    //     let tag_select =
    //         select("BookTag", "").filter(None, filter().add(None, ".kind = $:", "ㅁㅁ"));

    //     let r = insert("Book")
    //         .set(".title", "하안되겟네")
    //         .set_expr(".book_tags", tag_select)
    //         .on_conflict(do_nothing())
    //         .to_query();

    //     println!("{r}");
    // }
}
