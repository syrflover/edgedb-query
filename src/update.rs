use either::Either;

use super::*;

#[derive(Clone)]
pub struct UpdateBuilder<'a> {
    target: &'a str,
    withs: Vec<With<'a>>,
    filter: Option<Filter<'a>>,
    values: Vec<(&'a str, QueryArgOrExpr<'a>)>,
}

pub fn update(target: &str) -> UpdateBuilder {
    UpdateBuilder::new(target)
}

impl<'a> UpdateBuilder<'a> {
    pub fn new(target: &'a str) -> Self {
        Self {
            target,
            filter: None,
            values: Vec::new(),
            withs: Vec::new(),
        }
    }

    pub fn with(mut self, with: With<'a>) -> Self {
        self.withs.push(with);

        self
    }

    pub fn filter(mut self, filter: Filter<'a>) -> Self {
        self.filter.replace(filter);

        self
    }

    pub fn set<T>(mut self, field: &'a str, value: T) -> Self
    where
        T: ToQueryArg + 'a,
    {
        self.values.push((field, Either::Left(Box::new(value))));

        self
    }
}

impl<'a> ToQuery for UpdateBuilder<'a> {
    fn to_query_with_indent(&self, indent: usize) -> String {
        let mut qx = String::new();
        let q = &mut qx;

        // with
        {
            push_withs(q, &self.withs, indent);
        }

        // update
        {
            push_str(q, "update", indent);
            q.push(' ');

            q.push_str(self.target);
        }

        q.push('\n');

        // filter
        {
            push_filter(q, self.filter.as_ref(), indent);
        }

        q.push('\n');

        push_str(q, "set", indent);
        q.push(' ');

        // set values
        {
            push_object(q, &self.values, indent);
        }

        qx
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print() {
        let query = update("Book")
            .filter(filter().add(AND, ".uid = $:", 1234))
            .set("released", true)
            .to_query();

        println!("{query}");
    }
}
