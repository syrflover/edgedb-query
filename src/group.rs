use std::fmt::Debug;

use edgedb_protocol::queryable::Queryable;

use super::{push_str, ToQuery};

#[derive(Clone)]
pub struct GroupBuilder<'a> {
    target: &'a str,
    fields: &'a str,
    by: Vec<String>,
    using: Vec<String>,
}

pub fn group<'a>(target: &'a str, fields: &'a str) -> GroupBuilder<'a> {
    GroupBuilder::new(target, fields)
}

impl<'a> GroupBuilder<'a> {
    pub fn new(target: &'a str, fields: &'a str) -> Self {
        Self {
            target,
            fields,
            by: Vec::new(),
            using: Vec::new(),
        }
    }

    pub fn using(mut self, using: impl ToString) -> Self {
        self.using.push(using.to_string());

        self
    }

    pub fn by(mut self, by: impl ToString) -> Self {
        self.by.push(by.to_string());

        self
    }
}

impl<'a> ToQuery for GroupBuilder<'a> {
    fn to_query_with_indent(&self, indent: usize) -> String {
        let target = self.target;
        let fields = self.fields;

        let mut qx = String::new();
        let q = &mut qx;

        push_str(q, "group ", indent);
        q.push_str(target);

        if !fields.is_empty() {
            q.push_str(" { ");
            q.push_str(fields);
            q.push_str(" }");
        }

        // using

        if !self.using.is_empty() {
            q.push('\n');

            let using = self.using.join(", ");

            push_str(q, "using", indent);
            q.push('\n');
            push_str(q, &using, 2 + indent);
        }

        // by

        q.push('\n');

        let by = self.by.join(", ");

        push_str(q, "by", indent);

        q.push('\n');

        push_str(q, &by, 2 + indent);

        qx
    }
}

pub struct GroupResult<K, T> {
    pub key: K,
    pub grouping: Vec<String>,
    pub elements: Vec<T>,
}

impl<K, T> Debug for GroupResult<K, T>
where
    K: Debug,
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GroupResult")
            .field("key", &self.key)
            .field("grouping", &self.grouping)
            .field("elements", &self.elements)
            .finish()
    }
}

impl<K, T> Queryable for GroupResult<K, T>
where
    K: Queryable,
    T: Queryable,
{
    fn decode(
        decoder: &edgedb_protocol::queryable::Decoder,
        buf: &[u8],
    ) -> Result<Self, edgedb_protocol::errors::DecodeError> {
        let base_fields = 3;

        let nfields = base_fields
            + if decoder.has_implicit_id { 1 } else { 0 }
            + if decoder.has_implicit_tid { 1 } else { 0 }
            + if decoder.has_implicit_tname { 1 } else { 0 };
        let mut elems =
            edgedb_protocol::serialization::decode::DecodeTupleLike::new_object(buf, nfields)?;

        // type id block

        if decoder.has_implicit_id {
            elems.skip_element()?;
        }

        // type name block

        if decoder.has_implicit_tname {
            elems.skip_element()?;
        }

        // id block

        if decoder.has_implicit_id {
            elems.skip_element()?;
        }

        // field decoders

        macro_rules! field_decoders {
            ($(($ident:ident, $ty:ty)$(,)?)*) => {
                $(
                    let $ident: $ty =
                        edgedb_protocol::queryable::Queryable::decode_optional(
                            decoder,
                            elems.read()?,
                        )?;
                )*
            };
        }

        field_decoders![(key, K), (grouping, Vec<String>), (elements, Vec<T>)];

        // #type_id_block
        // #type_name_block
        // #id_block
        // #field_decoders
        // Ok(#name {
        //     #(
        //         #fieldname,
        //     )*
        // })

        Ok(GroupResult {
            key,
            grouping,
            elements,
        })
    }

    fn check_descriptor(
        ctx: &edgedb_protocol::queryable::DescriptorContext,
        type_pos: edgedb_protocol::descriptors::TypePos,
    ) -> Result<(), edgedb_protocol::queryable::DescriptorMismatch> {
        use ::edgedb_protocol::descriptors::Descriptor::ObjectShape;
        let desc = ctx.get(type_pos)?;
        let shape = match desc {
            ObjectShape(shape) => shape,
            _ => return Err(ctx.wrong_type(desc, "str")),
        };

        // TODO(tailhook) cache shape.id somewhere
        let mut idx = 0;

        if ctx.has_implicit_tid {
            if !shape.elements[idx].flag_implicit {
                return Err(ctx.expected("implicit __tid__"));
            }
            idx += 1;
        }

        if ctx.has_implicit_tname {
            if !shape.elements[idx].flag_implicit {
                return Err(ctx.expected("implicit __tname__"));
            }
            idx += 1;
        }

        if ctx.has_implicit_id {
            if !shape.elements[idx].flag_implicit {
                return Err(ctx.expected("implicit id"));
            }
            idx += 1;
        }

        macro_rules! field_checks {
            ($(($fieldname:expr, $fieldty:ty) $(,)?)*) => {
                $(
                    let el = &shape.elements[idx];
                    if el.name != $fieldname {
                        return Err(ctx.wrong_field($fieldname, &el.name));
                    }
                    idx += 1;
                    <$fieldty as ::edgedb_protocol::queryable::Queryable>::check_descriptor(
                        ctx,
                        el.type_pos,
                    )?;
                )*
            };
        }

        field_checks![
            ("key", K),
            ("grouping", Vec<String>),
            ("elements", Vec<T>),
        ];

        // #type_id_check
        // #type_name_check
        // #id_check
        // #field_checks

        if shape.elements.len() != idx {
            return Err(ctx.field_number(shape.elements.len(), idx));
        }

        Ok(())
    }
}
