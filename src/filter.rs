use either::Either;

use super::*;

#[derive(Clone, Copy)]
pub enum Not {
    Not,
}

pub const NOT: Not = Not::Not;

#[derive(Clone, Copy)]
pub enum AndOr {
    And,
    Or,
}

pub const AND: AndOr = AndOr::And;
pub const OR: AndOr = AndOr::Or;

impl AndOr {
    pub fn as_str(&self) -> &str {
        match self {
            AndOr::And => "and",
            AndOr::Or => "or",
        }
    }
}

// fn gt<'a>(x: &'a str, y: &'a str) {

// }

type Inner<'a> = (&'a str, Box<dyn ToQueryArg + 'a>);

#[derive(Clone)]
pub struct Filter<'a> {
    not: Option<Not>,
    func: Option<&'a str>,
    qs: Vec<(AndOr, Either<Inner<'a>, Filter<'a>>)>,
}

pub fn filter<'a>() -> Filter<'a> {
    Filter::new(None)
}

impl<'a> std::ops::Not for Filter<'a> {
    type Output = Self;

    fn not(mut self) -> Self::Output {
        self.not.replace(NOT);

        self
    }
}

impl<'a> Filter<'a> {
    pub fn new(prefix_operator: impl Into<Option<Not>>) -> Self {
        Self {
            not: prefix_operator.into(),
            func: None,
            qs: Vec::new(),
        }
    }

    pub fn func(mut self, func_name: &'a str) -> Self {
        self.func.replace(func_name);

        self
    }

    pub fn add(mut self, and_or: AndOr, q: &'a str, arg: impl ToQueryArg + 'a) -> Self {
        self.qs.push((and_or, (Either::Left((q, Box::new(arg))))));

        self
    }

    pub fn add_opt(self, and_or: AndOr, q: &'a str, arg: Option<impl ToQueryArg + 'a>) -> Self {
        if let Some(arg) = arg {
            self.add(and_or, q, arg)
        } else {
            self
        }
    }

    pub fn add_filter(mut self, and_or: AndOr, filter: Filter<'a>) -> Self {
        self.qs.push((and_or, Either::Right(filter)));

        self
    }

    pub fn is_empty(&self) -> bool {
        self.qs.is_empty()
    }

    fn to_query_internal(&self, indent: usize, wrapped_by_parentheses: bool) -> String {
        let mut qx = String::new();
        let q = &mut qx;

        let is_not = matches!(self.not, Some(Not::Not));
        let has_func = self.func.is_some();

        if is_not {
            push_str(q, "not", indent);
            q.push(' ');
        }

        if has_func {
            q.push_str(self.func.unwrap());
        }

        if is_not || has_func {
            q.push('(');
            q.push('\n');
        }

        let mut qs = self.qs.iter();

        if let Some((_, x)) = qs.next() {
            match x {
                Either::Left((x, arg)) => {
                    if wrapped_by_parentheses {
                        push(q, '(', 2 + indent);
                    } else {
                        push_str(q, "", 2 + indent);
                    }

                    q.push_str(&x.replacen(ARG_IDENTITY, &arg.to_query_arg(), 1));
                }
                Either::Right(x) => {
                    //
                    q.push_str(&x.to_query_internal(indent, true));
                }
            }
        }

        for (and_or, x) in qs {
            q.push(' ');
            q.push_str(and_or.as_str());

            q.push('\n');

            match x {
                Either::Left((x, arg)) => {
                    let x = x.replacen(ARG_IDENTITY, &arg.to_query_arg(), 1);
                    push_str(q, &x, 2 + indent);

                    if wrapped_by_parentheses {
                        q.push(')');
                    }
                }
                Either::Right(x) => {
                    //
                    q.push_str(&x.to_query_internal(indent, true));
                }
            }
        }

        if is_not || has_func {
            q.push('\n');
            push(q, ')', indent);
        }

        qx
    }
}

impl<'a> ToQuery for Filter<'a> {
    fn to_query_with_indent(&self, indent: usize) -> String {
        self.to_query_internal(indent, false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print() {
        let r = Filter::new(NOT)
            .func("any")
            .add(AND, ".uid = $:", 12345)
            .add(AND, ".title = $:", "arg")
            .add_filter(
                AND,
                Filter::new(None)
                    .add(AND, ".kind = $:", "arg")
                    .add(AND, ".name = $:", "arg"),
            )
            .to_query();

        println!("{r}");
    }
}
