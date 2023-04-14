use std::{borrow::Cow, ops::Deref};

use edgedb_protocol::{model::Datetime, value::Value as V};

#[derive(Clone)]
pub struct Value<'a> {
    pub inner: V,
    pub kind: Cow<'a, str>,
}

pub trait IntoValue<'a>: Send + Sync {
    fn into_value(self) -> Value<'a>;
}

impl<'a> IntoValue<'a> for Value<'a> {
    fn into_value(self) -> Value<'a> {
        self
    }
}

macro_rules! impl_tuple {
    ($($name:ident $(,)?)+) => {
        impl<'a, $($name,)+> IntoValue<'a> for ($($name,)+)
        where
            $(
                $name: IntoValue<'a>,
            )+
        {
            fn into_value(self) -> Value<'a> {
                #[allow(non_snake_case)]
                let ($($name,)+) = self;

                let (kinds, values): (Vec<_>, Vec<_>) = [
                    $(
                        $name.into_value(),
                    )+
                ]
                .into_iter()
                .map(|x| (x.kind, x.inner))
                .unzip();

                let kinds = kinds.join(", ");

                Value {
                    inner: V::Tuple(values),
                    kind: format!("tuple<{kinds}>").into()
                }

            }
        }
    };
}

impl_tuple![T1];
impl_tuple![T1, T2];
impl_tuple![T1, T2, T3];
impl_tuple![T1, T2, T3, T4];
impl_tuple![T1, T2, T3, T4, T5];
impl_tuple![T1, T2, T3, T4, T5, T6];
impl_tuple![T1, T2, T3, T4, T5, T6, T7];
impl_tuple![T1, T2, T3, T4, T5, T6, T7, T8];
impl_tuple![T1, T2, T3, T4, T5, T6, T7, T8, T9];
impl_tuple![T1, T2, T3, T4, T5, T6, T7, T8, T9, T10];
impl_tuple![T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11];
impl_tuple![T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12];

impl<'a, T> IntoValue<'a> for Vec<T>
where
    T: IntoValue<'a>,
{
    fn into_value(self) -> Value<'a> {
        let mut xs = self.into_iter().map(IntoValue::into_value);
        let Value { inner: x, kind } = xs.next().unwrap();

        let value = V::Set([x].into_iter().chain(xs.map(|x| x.inner)).collect());

        Value { inner: value, kind }
    }
}

impl<'a> IntoValue<'a> for String {
    fn into_value(self) -> Value<'a> {
        Value {
            inner: V::Str(self),
            kind: "str".into(),
        }
    }
}

impl<'a> IntoValue<'a> for &String {
    fn into_value(self) -> Value<'a> {
        self.deref().into_value()
    }
}

impl<'a> IntoValue<'a> for &str {
    fn into_value(self) -> Value<'a> {
        self.to_string().into_value()
    }
}

impl<'a> IntoValue<'a> for bool {
    fn into_value(self) -> Value<'a> {
        let value = V::Bool(self);

        Value {
            kind: value.kind().into(),
            inner: value,
        }
    }
}

impl<'a> IntoValue<'a> for i8 {
    fn into_value(self) -> Value<'a> {
        let value = V::Int16(self as i16);

        Value {
            kind: value.kind().into(),
            inner: value,
        }
    }
}

impl<'a> IntoValue<'a> for i16 {
    fn into_value(self) -> Value<'a> {
        let value = V::Int16(self);

        Value {
            kind: value.kind().into(),
            inner: value,
        }
    }
}

impl<'a> IntoValue<'a> for i32 {
    fn into_value(self) -> Value<'a> {
        let value = V::Int32(self);

        Value {
            kind: value.kind().into(),
            inner: value,
        }
    }
}

impl<'a> IntoValue<'a> for i64 {
    fn into_value(self) -> Value<'a> {
        let value = V::Int64(self);

        Value {
            kind: value.kind().into(),
            inner: value,
        }
    }
}

impl<'a> IntoValue<'a> for u8 {
    fn into_value(self) -> Value<'a> {
        let value = V::Int16(self as i16);

        Value {
            kind: value.kind().into(),
            inner: value,
        }
    }
}

impl<'a> IntoValue<'a> for u16 {
    fn into_value(self) -> Value<'a> {
        let value = V::Int16(self as i16);

        Value {
            kind: value.kind().into(),
            inner: value,
        }
    }
}

impl<'a> IntoValue<'a> for u32 {
    fn into_value(self) -> Value<'a> {
        let value = V::Int32(self as i32);

        Value {
            kind: value.kind().into(),
            inner: value,
        }
    }
}

impl<'a> IntoValue<'a> for u64 {
    fn into_value(self) -> Value<'a> {
        let value = V::Int64(self as i64);

        Value {
            kind: value.kind().into(),
            inner: value,
        }
    }
}

// impl<'a> IntoValue<'a> for DateTime<Utc> {
//     fn into_value(self) -> Value<'a> {
//         Value::Datetime(self.try_into().unwrap())
//     }
// }

impl<'a> IntoValue<'a> for Datetime {
    fn into_value(self) -> Value<'a> {
        let value = V::Datetime(self);

        Value {
            kind: value.kind().into(),
            inner: value,
        }
    }
}
