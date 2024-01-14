use std::borrow::Cow;

use super::{push_str, ToQuery};

// TODO: multiple argments 지원
// 현재는 $?를 통해 인자 하나만 받을 수 있도록 구현돼있음

#[derive(Clone)]
pub struct OrderBy<'a> {
    by: Cow<'a, str>,
    direction: Option<OrderDirection>,
}

pub fn order_by<'a>(
    by: impl Into<Cow<'a, str>>,
    direction: impl Into<Option<OrderDirection>>,
) -> OrderBy<'a> {
    OrderBy::new(by.into(), direction)
}

impl<'a> OrderBy<'a> {
    pub fn new(by: Cow<'a, str>, direction: impl Into<Option<OrderDirection>>) -> Self {
        Self {
            by,
            direction: direction.into(),
        }
    }
}

impl<'a> ToQuery for OrderBy<'a> {
    fn to_query_with_indent(&self, indent: usize) -> String {
        let mut qx = String::new();
        let q = &mut qx;

        push_str(q, &self.by, 2 + indent);

        if let Some(direction) = self.direction {
            q.push(' ');
            q.push_str(direction.as_str());
        }

        qx
    }
}

#[derive(Clone, Copy)]
pub enum OrderDirection {
    Desc,
    Asc,
}

pub const DESC: OrderDirection = OrderDirection::Desc;
pub const ASC: OrderDirection = OrderDirection::Asc;

impl OrderDirection {
    pub(super) fn as_str(&self) -> &str {
        match self {
            OrderDirection::Desc => "desc",
            OrderDirection::Asc => "asc",
        }
    }
}
