
use anchor_lang::prelude::Pubkey;

use crate::types::*;




impl ResourceSchema {
    pub fn encode(&self) -> ResourceType {
        pub fn _encode(node: &ResourceSchema, buf: &mut Vec<u8>) {
            let write_str = |buf: &mut Vec<u8>, s: &String| {
                buf.push(s.len() as u8);
                buf.extend_from_slice(s.as_bytes());
            };
            match &node.0 {
                RS::U8 => { buf.push(0) }
                RS::U16 => { buf.push(1) }
                RS::U32 => { buf.push(2) }
                RS::U64 => { buf.push(3) }
                RS::U128 => { buf.push(4) }
                RS::Bool => { buf.push(5) }
                RS::String => { buf.push(6) }
                RS::Buffer => { buf.push(7) }
                RS::Pubkey => { buf.push(8) },
                RS::Option(val) => {
                    buf.push(9);
                    _encode(val, buf);
                }
                RS::List(val) => {
                    buf.push(10);
                    _encode(val, buf);
                }
                RS::Map(val) => {
                    buf.push(11);
                    _encode(val, buf);
                }
                RS::Struct(fields) => {
                    buf.push(12);
                    buf.push(fields.len() as u8);
                    for (k, val) in fields {
                        write_str(buf, k);
                        _encode(val, buf);
                    };
                }
            }
        }
        let mut v = Vec::with_capacity(100);
        _encode(self, &mut v);
        ResourceType(v)
    }

    pub fn decode(def: &ResourceType) -> std::result::Result<ResourceSchema, String> {
        Self::_decode(&mut &*def.0)
    }

    pub(crate) fn _decode(buf: &mut &[u8]) -> std::result::Result<ResourceSchema, String> {
        fn unshift(buf: &mut &[u8]) -> u8 {
            let out = buf[0];
            *buf = &buf[1..];
            out
        }
        let idx = unshift(buf);
        Ok(ResourceSchema(
            match idx {
                0 => RS::U8,
                1 => RS::U16,
                2 => RS::U32,
                3 => RS::U64,
                4 => RS::U128,
                5 => RS::Bool,
                6 => RS::String,
                7 => RS::Buffer,
                8 => RS::Pubkey,
                9 => RS::Option(Self::_decode(buf)?.into()),
                10 => RS::List(Self::_decode(buf)?.into()),
                11 => RS::Map(Self::_decode(buf)?.into()),
                12 => {
                    let num_fields = unshift(buf) as usize;
                    let mut fields: Vec<(String, ResourceSchema)> = Vec::with_capacity(num_fields);
                    for _ in 0..num_fields {
                        let key_len = unshift(buf) as usize;
                        let (kb, rest) = buf.split_at(key_len);
                        *buf = rest;
                        let key = std::str::from_utf8(kb).map_err(|op| op.to_string())?;
                        let val = Self::_decode(buf)?.into();
                        fields.push((key.to_string(), val));
                    }
                    RS::Struct(fields)
                },
                _ => { return Err("Unmatched tag".to_string()); }
            }
        ))
    }
}

impl ResourceSchema {
    pub fn to_string(&self) -> String {
        return format!("{:?}", self);
        //let mut out = "".to_owned();

        //let mut inner = |s: &RS| {
        //};
        //inner(&self.0);
        //out
    }

    pub fn new_struct<V: Into<ResourceSchema>>(items: Vec<(&str, V)>) -> ResourceSchema {
        // validate
        for i in items.windows(2) {
            assert!(i[0].0 < i[1].0, "ResourceSchema::new_struct: keys out of order or dupe");
        }
        RS::Struct(items.into_iter().map(|(k,v)| item(k, v)).collect()).into()
    }
}



impl Into<ResourceSchema> for RS {
    fn into(self) -> ResourceSchema {
        ResourceSchema(self)
    }
}
impl Into<Box<ResourceSchema>> for RS {
    fn into(self) -> Box<ResourceSchema> {
        Box::new(ResourceSchema(self))
    }
}

macro_rules! impl_is_resource_data {
    ($type:path, $item:ident) => {
        impl IsResourceData for $type {
            fn to_resource_schema() -> ResourceSchema {
                RS::$item.into()
            }
        }
        impl Into<RD> for $type {
            fn into(self) -> RD {
                RD::$item(self)
            }
        }
    };
}
macro_rules! impl_is_resource_data_complex {
    ($type:tt, $item:ident) => {
        impl<T: IsResourceData> IsResourceData for $type<T> {
            fn to_resource_schema() -> ResourceSchema {
                RS::$item(T::to_resource_schema().into()).into()
            }
        }
    };
}

impl_is_resource_data!(u8, U8);
impl_is_resource_data!(u16, U16);
impl_is_resource_data!(u32, U32);
impl_is_resource_data!(u64, U64);
impl_is_resource_data!(u128, U128);
impl_is_resource_data!(bool, Bool);
impl_is_resource_data!(String, String);
impl_is_resource_data!(Buffer, Buffer);
impl_is_resource_data!(Pubkey, Pubkey);
impl_is_resource_data_complex!(Option, Option);
impl<T: IsResourceData> Into<RD> for Option<T> {
    fn into(self) -> RD {
        RD::Option(self.map(|o| o.into()).into())
    }
}
impl_is_resource_data_complex!(Vec, List);
impl<T: IsResourceData> Into<RD> for Vec<T> {
    fn into(self) -> RD {
        RD::List(self.into_iter().map(|o| o.into()).collect())
    }
}



pub fn schema_is_superset(subset: &RS, superset: &RS) -> bool {
    match (subset, superset) {
        (RS::Option(sub_inner), RS::Option(super_inner)) => schema_is_superset(sub_inner, super_inner),
        (RS::List(a_inner), RS::List(b_inner)) => schema_is_superset(a_inner, b_inner),
        (RS::Map(a_inner), RS::Map(b_inner)) => schema_is_superset(a_inner, b_inner),
        (RS::Struct(a_items), RS::Struct(b_items)) => {
            let mut b = 0;
            for (key, val) in a_items {
                for (k2, v2) in b_items[b..].iter() {
                    b += 1;
                    if key > k2 {
                        continue;
                    }
                    if key == k2 {
                        if !schema_is_superset(val, v2) {
                            return false;
                        }
                        break;
                    } else {
                        return false;
                    }
                }
            }
            true
        },
        (a, b) => a == b
    }
}


pub fn validate_resource_data<S: ToString>(key: &S, schema: &RS, data: &RD) -> std::result::Result<(), String> {
    let r = match (schema, data) {
        (RS::U8, RD::U8(_)) => Ok(()),
        (RS::U16, RD::U16(_)) => Ok(()),
        (RS::U32, RD::U32(_)) => Ok(()),
        (RS::U64, RD::U64(_)) => Ok(()),
        (RS::U128, RD::U128(_)) => Ok(()),
        (RS::Bool, RD::Bool(_)) => Ok(()),
        (RS::String, RD::String(_)) => Ok(()),
        (RS::Buffer, RD::Buffer(_)) => Ok(()),
        (RS::Pubkey, RD::Pubkey(_)) => Ok(()),
        (RS::Option(inner_schema), RD::Option(inner_data)) => {
            if let Some(inner) = &**inner_data {
                validate_resource_data(&"0", &**inner_schema, inner)
            } else {
                Ok(())
            }
        }
        (RS::List(item_schema), RD::List(data_items)) => {
            data_items.iter().enumerate().map(|(i,data_item)|
                validate_resource_data(&i, item_schema, data_item)
            ).collect()
        }
        (RS::Map(schema), RD::Map(items)) => {
            items.iter().map(|(k,i)| validate_resource_data(k, schema, &*i)).collect()
        }
        (RS::Struct(schema_fields), RD::Struct(data_fields)) => {
            // We want the struct to be upgradeable, so we need a way to have empty fields here
            // but we'll do that later

            if schema_fields.len() != data_fields.len() {
                Err("Struct wrong length".to_string())
            } else {
                schema_fields.iter().zip(data_fields).map(|(schema, data)| {
                    if schema.0 != data.0 {
                        Err("Struct wrong key".to_string())
                    } else {
                        validate_resource_data(&schema.0, &schema.1, &data.1)
                    }
                }).collect()
            }
        }
        _ => Err("Schema mismatch".to_string()),
    };

    r.map_err(|s| {
        let k = key.to_string();
        if k.is_empty() { s } else { format!("{}: {}", k, s) }
    })
}

fn item<T, V: Into<T>>(k: &str, v: V) -> (String, T) {
    (k.into(), v.into())
}




#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_simple() {
        let r = validate_resource_data(&"", &RS::U8, &RD::U8(1));
        assert!(r == Ok(()));
    }

    #[test]
    fn test_validate_error() {
        let r = validate_resource_data(
            &"root",
            &RS::List(RS::U16.into()),
            &RD::List(vec![RD::U16(1).into(), RD::U8(1).into()]),
        );
        assert!(r == Err("root: 1: Schema mismatch".to_string()));
    }

    #[test]
    fn test_validate_struct() {
        let schema = ResourceSchema::new_struct(vec![
            ("age", RS::U8),
            ("name", RS::String),
            ("things", RS::Map(RS::Bool.into())),
        ]);

        let data = RD::Struct(vec![
            item("age", RD::U8(30)),
            item("name", RD::String("abc".into())),
            item("things", RD::Map(vec![
                item("chicken", true),
                item("cow", true),
                item("fox", false)
            ].into_iter().collect()))
        ]);

        let r = validate_resource_data(&"root", &schema, &data);
        //println!("{:?}", r);
        assert!(Ok(()) == r);
    }

    #[test]
    fn test_serialize_schema() {
        let schema = ResourceSchema::new_struct(vec![
            ("age", RS::U8),
            ("name", RS::String),
            ("things", RS::Map(RS::Bool.into())),
        ]);

        //println!("Debug: {}", schema.to_string());
        //println!("Debug: {}", schema.to_string().len());
        //println!("borsh: {:?}", schema.try_to_vec().unwrap());
        //println!("borsh: {:?}", schema.try_to_vec().unwrap().len());
        let buf = schema.encode();
        //println!("encode: {:?}", buf);
        //println!("encode: {:?}", buf.len());

        let r = ResourceSchema::decode(&buf).unwrap();
        //println!("Debug: {}", r.to_string());

        assert!(schema == r);
    }

    #[test]
    fn test_superset() {
        let fields = vec![
            ("age", RS::U8),
            ("name", RS::String),
            ("things", RS::Map(RS::Bool.into())),
        ];
        let old = RS::List(ResourceSchema::new_struct(fields.clone()[1..].to_vec()).into());
        let new = RS::List(ResourceSchema::new_struct(fields.clone()).into());

        assert!(schema_is_superset(&old, &old));
        assert!(schema_is_superset(&old, &new));
        assert!(!schema_is_superset(&new, &old));
    }

    use quickcheck::{Arbitrary, Gen};

    impl Arbitrary for ResourceSchema {
        fn arbitrary(g: &mut Gen) -> ResourceSchema {
            let v = Vec::from_iter(0..14);
            let idx = g.choose(&v).unwrap();
            ResourceSchema(
                match idx {
                    0 => RS::U8,
                    1 => RS::U16,
                    2 => RS::U32,
                    3 => RS::U64,
                    4 => RS::U128,
                    5 => RS::Bool,
                    6 => RS::String,
                    7 => RS::Buffer,
                    8 => RS::Pubkey,
                    9 => RS::Option(ResourceSchema::arbitrary(g).into()),
                    10 => RS::List(ResourceSchema::arbitrary(g).into()),
                    11 => RS::Map(ResourceSchema::arbitrary(g).into()),
                    _ => {
                        let n = *g.choose(&[1,2,3,4,5]).unwrap();
                        RS::Struct(
                            (0..n).map(|i| (i.to_string(), ResourceSchema::arbitrary(g))).collect()
                        )
                    },
                }
            )
        }
    }

    #[quickcheck]
    fn identity_is_superset(schema: ResourceSchema) -> bool {
        //println!("schema is: {:?}", schema);
        schema_is_superset(&*schema, &*schema)
    }

    #[quickcheck]
    fn encoding_identity(schema: ResourceSchema) -> bool {
        let encoded = schema.encode();
        //println!("{:?}: {:?}", encoded, schema);
        schema == ResourceSchema::decode(&encoded).unwrap()
    }
}
