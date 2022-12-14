use rust_decimal::Decimal;
use serde::ser::Impossible;

use crate::{Error, Map, Value};

pub struct Serializer;

impl serde::Serializer for Serializer {
    type Ok = Value;
    type Error = Error;

    type SerializeSeq = SerializeVec;
    type SerializeTuple = SerializeVec;
    type SerializeTupleStruct = SerializeVec;
    type SerializeTupleVariant = SerializeTupleVariant;
    type SerializeMap = SerializeMap;
    type SerializeStruct = SerializeMap;
    type SerializeStructVariant = SerializeStructVariant;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Bool(v))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(Value::I8(v))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(Value::I16(v))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(Value::I32(v))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(Value::I64(v))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(Value::U8(v))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(Value::U16(v))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(Value::U32(v))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(Value::U64(v))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(Value::F32(v))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(Value::F64(v))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Ok(Value::String(String::from(v)))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(Value::String(v.to_owned()))
    }

    fn serialize_bytes(self, _: &[u8]) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Null)
    }

    fn serialize_some<T>(self, v: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        v.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Null)
    }

    fn serialize_unit_struct(self, _: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(self, _: &'static str, v: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        v.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(SerializeVec {
            vec: Vec::with_capacity(len.unwrap_or(0)),
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(SerializeMap::Map {
            map: Map::with_capacity(len.unwrap_or(0)),
            next_key: None,
        })
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        use crate::tokens;
        let token = match name {
            tokens::DECIMAL => Type::Decimal,
            tokens::PUBKEY => Type::Pubkey,
            tokens::KEYPAIR => Type::Keypair,
            tokens::SIGNATURE => Type::Signature,
            _ => {
                return Ok(SerializeMap::Map {
                    map: Map::with_capacity(len),
                    next_key: None,
                })
            }
        };
        Ok(SerializeMap::Type {
            token,
            out_value: None,
        })
    }

    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(SerializeStructVariant {
            name: variant.to_owned(),
            map: Map::with_capacity(len),
        })
    }

    fn serialize_newtype_variant<T>(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        Ok(Value::Map(Map::from([(
            variant.to_owned(),
            value.serialize(Serializer)?,
        )])))
    }

    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(SerializeTupleVariant {
            name: variant.to_owned(),
            vec: Vec::with_capacity(len),
        })
    }
}

pub struct SerializeTupleVariant {
    name: String,
    vec: Vec<Value>,
}

impl serde::ser::SerializeTupleVariant for SerializeTupleVariant {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.vec.push(value.serialize(Serializer)?);
        Ok(())
    }

    fn end(self) -> Result<Value, Error> {
        Ok(Value::Map(Map::from([(self.name, Value::Array(self.vec))])))
    }
}

pub struct SerializeStructVariant {
    name: String,
    map: Map,
}

impl serde::ser::SerializeStructVariant for SerializeStructVariant {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.map
            .insert(key.to_owned(), value.serialize(Serializer)?);
        Ok(())
    }

    fn end(self) -> Result<Value, Error> {
        Ok(Value::Map(Map::from([(self.name, Value::Map(self.map))])))
    }
}

pub struct SerializeVec {
    vec: Vec<Value>,
}

impl serde::ser::SerializeSeq for SerializeVec {
    type Ok = Value;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.vec.push(value.serialize(Serializer)?);
        Ok(())
    }

    fn end(self) -> Result<Value, Error> {
        Ok(Value::Array(self.vec))
    }
}

impl serde::ser::SerializeTuple for SerializeVec {
    type Ok = Value;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ?Sized + serde::Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Value, Error> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl serde::ser::SerializeTupleStruct for SerializeVec {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ?Sized + serde::Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Value, Error> {
        serde::ser::SerializeSeq::end(self)
    }
}

pub enum SerializeMap {
    Map {
        map: Map,
        next_key: Option<String>,
    },
    Type {
        token: Type,
        out_value: Option<Value>,
    },
}

#[derive(Clone, Copy)]
pub enum Type {
    Decimal,
    Pubkey,
    Keypair,
    Signature,
}

impl serde::ser::SerializeMap for SerializeMap {
    type Ok = Value;
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Error>
    where
        T: ?Sized + serde::Serialize,
    {
        match self {
            SerializeMap::Map { next_key, .. } => {
                *next_key = Some(key.serialize(MapKeySerializer)?);
                Ok(())
            }
            SerializeMap::Type { .. } => unreachable!(),
        }
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ?Sized + serde::Serialize,
    {
        match self {
            SerializeMap::Map { map, next_key } => {
                let key = next_key.take();
                let key = key.expect("serialize_value called before serialize_key");
                map.insert(key, value.serialize(Serializer)?);
                Ok(())
            }
            SerializeMap::Type { .. } => unreachable!(),
        }
    }

    fn end(self) -> Result<Value, Error> {
        match self {
            SerializeMap::Map { map, .. } => Ok(Value::Map(map)),
            SerializeMap::Type { .. } => unreachable!(),
        }
    }
}

impl serde::ser::SerializeStruct for SerializeMap {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Error>
    where
        T: ?Sized + serde::Serialize,
    {
        match self {
            SerializeMap::Map { .. } => serde::ser::SerializeMap::serialize_entry(self, key, value),
            SerializeMap::Type { token, out_value } => {
                *out_value = Some(value.serialize(CustomTypeEmitter { token: *token })?);
                Ok(())
            }
        }
    }

    fn end(self) -> Result<Value, Error> {
        match self {
            SerializeMap::Map { .. } => serde::ser::SerializeMap::end(self),
            SerializeMap::Type { out_value, .. } => {
                Ok(out_value.expect("serialize_field not called"))
            }
        }
    }
}

struct CustomTypeEmitter {
    token: Type,
}

impl serde::Serializer for CustomTypeEmitter {
    type Ok = Value;
    type Error = Error;

    type SerializeSeq = Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;
    type SerializeMap = Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        unreachable!();
    }

    fn serialize_newtype_struct<T>(self, _: &'static str, _: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        unreachable!();
    }

    fn serialize_bool(self, _: bool) -> Result<Self::Ok, Self::Error> {
        unreachable!();
    }

    fn serialize_i8(self, _: i8) -> Result<Self::Ok, Self::Error> {
        unreachable!();
    }

    fn serialize_i16(self, _: i16) -> Result<Self::Ok, Self::Error> {
        unreachable!();
    }

    fn serialize_i32(self, _: i32) -> Result<Self::Ok, Self::Error> {
        unreachable!();
    }

    fn serialize_i64(self, _: i64) -> Result<Self::Ok, Self::Error> {
        unreachable!();
    }

    fn serialize_u8(self, _: u8) -> Result<Self::Ok, Self::Error> {
        unreachable!();
    }

    fn serialize_u16(self, _: u16) -> Result<Self::Ok, Self::Error> {
        unreachable!();
    }

    fn serialize_u32(self, _: u32) -> Result<Self::Ok, Self::Error> {
        unreachable!();
    }

    fn serialize_u64(self, _: u64) -> Result<Self::Ok, Self::Error> {
        unreachable!();
    }

    fn serialize_f32(self, _: f32) -> Result<Self::Ok, Self::Error> {
        unreachable!();
    }

    fn serialize_f64(self, _: f64) -> Result<Self::Ok, Self::Error> {
        unreachable!();
    }

    fn serialize_char(self, _: char) -> Result<Self::Ok, Self::Error> {
        unreachable!();
    }

    fn serialize_str(self, _: &str) -> Result<Self::Ok, Self::Error> {
        unreachable!();
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<Self::Ok, Self::Error> {
        match self.token {
            Type::Decimal => {
                assert_eq!(value.len(), 16);
                let buf: [u8; 16] = value.try_into().unwrap();
                Ok(Value::Decimal(Decimal::deserialize(buf)))
            }
            Type::Pubkey => {
                assert_eq!(value.len(), solana_sdk::pubkey::PUBKEY_BYTES);
                let buf: [u8; solana_sdk::pubkey::PUBKEY_BYTES] = value.try_into().unwrap();
                Ok(Value::Pubkey(solana_sdk::pubkey::Pubkey::new_from_array(
                    buf,
                )))
            }
            Type::Keypair => {
                assert_eq!(value.len(), ed25519_dalek::KEYPAIR_LENGTH);
                let buf: [u8; ed25519_dalek::KEYPAIR_LENGTH] = value.try_into().unwrap();
                Ok(Value::Keypair(buf))
            }
            Type::Signature => {
                assert_eq!(value.len(), solana_sdk::signature::SIGNATURE_BYTES);
                Ok(Value::Signature(solana_sdk::signature::Signature::new(
                    value,
                )))
            }
        }
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        unreachable!();
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        unreachable!();
    }

    fn serialize_newtype_variant<T>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        unreachable!();
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        unreachable!();
    }

    fn serialize_some<T>(self, _: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        unreachable!();
    }

    fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        unreachable!();
    }

    fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, Self::Error> {
        unreachable!();
    }

    fn serialize_tuple_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        unreachable!();
    }

    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        unreachable!();
    }

    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        unreachable!();
    }

    fn serialize_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        unreachable!();
    }

    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        unreachable!();
    }
}

struct MapKeySerializer;

fn key_must_be_a_string() -> Error {
    Error::KeyMustBeAString
}

impl serde::Serializer for MapKeySerializer {
    type Ok = String;
    type Error = Error;

    type SerializeSeq = Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;
    type SerializeMap = Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<String, Error> {
        Ok(variant.to_owned())
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<String, Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_bool(self, _value: bool) -> Result<String, Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_i8(self, value: i8) -> Result<String, Error> {
        Ok(value.to_string())
    }

    fn serialize_i16(self, value: i16) -> Result<String, Error> {
        Ok(value.to_string())
    }

    fn serialize_i32(self, value: i32) -> Result<String, Error> {
        Ok(value.to_string())
    }

    fn serialize_i64(self, value: i64) -> Result<String, Error> {
        Ok(value.to_string())
    }

    fn serialize_u8(self, value: u8) -> Result<String, Error> {
        Ok(value.to_string())
    }

    fn serialize_u16(self, value: u16) -> Result<String, Error> {
        Ok(value.to_string())
    }

    fn serialize_u32(self, value: u32) -> Result<String, Error> {
        Ok(value.to_string())
    }

    fn serialize_u64(self, value: u64) -> Result<String, Error> {
        Ok(value.to_string())
    }

    fn serialize_f32(self, _value: f32) -> Result<String, Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_f64(self, _value: f64) -> Result<String, Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_char(self, value: char) -> Result<String, Error> {
        Ok({
            let mut s = String::new();
            s.push(value);
            s
        })
    }

    fn serialize_str(self, value: &str) -> Result<String, Error> {
        Ok(value.to_owned())
    }

    fn serialize_bytes(self, _value: &[u8]) -> Result<String, Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_unit(self) -> Result<String, Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<String, Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<String, Error>
    where
        T: ?Sized + serde::Serialize,
    {
        Err(key_must_be_a_string())
    }

    fn serialize_none(self) -> Result<String, Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_some<T>(self, _value: &T) -> Result<String, Error>
    where
        T: ?Sized + serde::Serialize,
    {
        Err(key_must_be_a_string())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Error> {
        Err(key_must_be_a_string())
    }

    fn collect_str<T>(self, value: &T) -> Result<String, Error>
    where
        T: ?Sized + std::fmt::Display,
    {
        Ok(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;
    use solana_sdk::pubkey::Pubkey;
    use solana_sdk::signature::Signature;
    use solana_sdk::signer::keypair::Keypair;
    use std::collections::HashMap;

    fn s<T: Serialize>(t: T) -> Value {
        t.serialize(Serializer).unwrap()
    }

    #[test]
    fn test_serialize_primitive() {
        assert_eq!(s(0u8), Value::U8(0));
        assert_eq!(s(0u16), Value::U16(0));
        assert_eq!(s(0u32), Value::U32(0));
        assert_eq!(s(0u64), Value::U64(0));
        assert_eq!(s(0i8), Value::I8(0));
        assert_eq!(s(0i16), Value::I16(0));
        assert_eq!(s(0i32), Value::I32(0));
        assert_eq!(s(0i64), Value::I64(0));
        assert_eq!(s(0f32), Value::F32(0.0));
        assert_eq!(s(0f64), Value::F64(0.0));
        assert_eq!(s(true), Value::Bool(true));
        assert_eq!(s(Option::<()>::None), Value::Null);
        assert_eq!(s(()), Value::Null);
        assert_eq!(s(()), Value::Null);
        assert_eq!(s("end"), Value::String("end".to_owned()));
        assert_eq!(
            s((1i32, 2i32, "hello")),
            Value::Array(
                [
                    Value::I32(1),
                    Value::I32(2),
                    Value::String("hello".to_owned())
                ]
                .to_vec()
            )
        );
        assert_eq!(s([0u8; 0]), Value::Array(Vec::new()));
        assert_eq!(s([()]), Value::Array([Value::Null].to_vec()));
        assert_eq!(
            s(HashMap::from([("a".to_owned(), -1i32)])),
            Value::Map(Map::from([("a".to_owned(), Value::I32(-1i32))]))
        );
    }

    #[test]
    fn test_derive() {
        #[derive(Serialize)]
        struct A {
            a: Noop,
            b: B,
            c0: C,
            c1: C,
            #[serde(flatten)]
            f: C,
            #[serde(rename = "NULL")]
            null: Option<i32>,
            i: I32,
        }

        #[derive(Serialize)]
        struct Noop;

        #[derive(Serialize)]
        struct B {}

        #[derive(Serialize)]
        struct I32(i32);

        #[derive(Serialize)]
        struct C {
            #[serde(skip_serializing_if = "Option::is_none")]
            y: Option<i32>,
        }

        assert_eq!(
            s(A {
                a: Noop,
                b: B {},
                c0: C { y: None },
                c1: C { y: Some(1) },
                f: C { y: Some(2) },
                null: None,
                i: I32(323232),
            }),
            Value::Map(
                [
                    ("a".into(), Value::Null),
                    ("b".into(), Value::Map(Map::new())),
                    ("c0".into(), Value::Map(Map::new())),
                    (
                        "c1".into(),
                        Value::Map([("y".into(), Value::I32(1))].into())
                    ),
                    ("y".into(), Value::I32(2)),
                    ("NULL".into(), Value::Null),
                    ("i".into(), Value::I32(323232)),
                ]
                .into()
            )
        );
    }

    #[test]
    fn test_enum() {
        #[derive(Serialize, Debug, PartialEq)]
        enum Enum0 {
            V0,
            V1,
            #[serde(rename = "var")]
            V2,
            V3(i32),
        }
        assert_eq!(s(Enum0::V0), Value::String("V0".to_owned()));
        assert_eq!(s(Enum0::V1), Value::String("V1".to_owned()));
        assert_eq!(s(Enum0::V2), Value::String("var".to_owned()));
        assert_eq!(
            s(Enum0::V3(1)),
            Value::Map(Map::from([("V3".to_owned(), Value::I32(1))]))
        );
    }

    #[test]
    fn test_custom_types() {
        #[derive(Serialize)]
        struct A {
            #[serde(with = "crate::decimal")]
            d: Decimal,
            #[serde(with = "crate::pubkey")]
            p: Pubkey,
            #[serde(with = "crate::keypair")]
            k: Keypair,
            #[serde(with = "crate::signature")]
            s: Signature,
        }

        assert_eq!(
            s(A {
                d: Decimal::ZERO,
                p: Pubkey::new_from_array([6; 32]),
                k: Keypair::from_bytes(&[3; 64]).unwrap(),
                s: Signature::new(&[4; 64]),
            }),
            Value::Map(
                [
                    ("d".to_owned(), Value::Decimal(Decimal::ZERO)),
                    (
                        "p".to_owned(),
                        Value::Pubkey(Pubkey::new_from_array([6; 32]))
                    ),
                    ("k".to_owned(), Value::Keypair([3; 64])),
                    ("s".to_owned(), Value::Signature(Signature::new(&[4; 64])))
                ]
                .into()
            )
        );
    }
}
