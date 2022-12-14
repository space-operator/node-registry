use crate::Map;
use crate::{Error, Value};
use rust_decimal::{prelude::ToPrimitive, Decimal};
use serde::de::{DeserializeSeed, Error as _, IntoDeserializer, Visitor};
use std::borrow::Cow;

impl<'de> serde::Deserializer<'de> for Value {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Null => visitor.visit_unit(),
            Value::String(s) => visitor.visit_string(s),
            Value::Bool(b) => visitor.visit_bool(b),
            Value::I8(i) => visitor.visit_i8(i),
            Value::I16(i) => visitor.visit_i16(i),
            Value::I32(i) => visitor.visit_i32(i),
            Value::I64(i) => visitor.visit_i64(i),
            Value::U8(u) => visitor.visit_u8(u),
            Value::U16(u) => visitor.visit_u16(u),
            Value::U32(u) => visitor.visit_u32(u),
            Value::U64(u) => visitor.visit_u64(u),
            Value::F32(f) => visitor.visit_f32(f),
            Value::F64(f) => visitor.visit_f64(f),
            Value::Decimal(d) => visit_decimal(d, visitor),
            Value::U128(u) => visitor.visit_u128(u),
            Value::Pubkey(p) => visitor.visit_bytes(p.as_ref()),
            Value::Keypair(k) => visitor.visit_bytes(&k),
            Value::Signature(s) => visitor.visit_bytes(s.as_ref()),
            Value::Array(array) => visit_array(array, visitor),
            Value::Map(map) => visit_map(map, visitor),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Null => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        use crate::tokens;
        match name {
            tokens::DECIMAL => match self {
                Value::Decimal(d) => visitor.visit_bytes(&d.serialize()),
                Value::I8(i) => visitor.visit_i8(i),
                Value::I16(i) => visitor.visit_i16(i),
                Value::I32(i) => visitor.visit_i32(i),
                Value::I64(i) => visitor.visit_i64(i),
                Value::U8(u) => visitor.visit_u8(u),
                Value::U16(u) => visitor.visit_u16(u),
                Value::U32(u) => visitor.visit_u32(u),
                Value::U64(u) => visitor.visit_u64(u),
                Value::F32(f) => visitor.visit_f32(f),
                Value::F64(f) => visitor.visit_f64(f),
                Value::String(s) => visitor.visit_string(s),
                _ => Err(serde::de::Error::invalid_type(
                    self.unexpected(),
                    &"decimal",
                )),
            },
            tokens::PUBKEY => match self {
                Value::Pubkey(p) => visitor.visit_bytes(p.as_ref()),
                Value::Keypair(k) => visitor.visit_bytes(&k),
                Value::String(s) => visitor.visit_str(&s),
                _ => Err(serde::de::Error::invalid_type(self.unexpected(), &"pubkey")),
            },
            tokens::KEYPAIR => match self {
                Value::Keypair(k) => visitor.visit_bytes(&k),
                Value::String(s) => visitor.visit_str(&s),
                _ => Err(serde::de::Error::invalid_type(
                    self.unexpected(),
                    &"keypair",
                )),
            },
            tokens::SIGNATURE => match self {
                Value::Signature(s) => visitor.visit_bytes(s.as_ref()),
                Value::String(s) => visitor.visit_str(&s),
                _ => Err(serde::de::Error::invalid_type(
                    self.unexpected(),
                    &"signature",
                )),
            },
            _ => visitor.visit_newtype_struct(self),
        }
    }

    fn deserialize_enum<V>(
        self,
        _: &'static str,
        _: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let (variant, value) = match self {
            Value::Map(value) => {
                let mut iter = value.into_iter();
                let (variant, value) = match iter.next() {
                    Some(v) => v,
                    None => {
                        return Err(serde::de::Error::invalid_value(
                            serde::de::Unexpected::Map,
                            &"map with a single key",
                        ));
                    }
                };
                // enums are encoded in json as maps with a single key:value pair
                if iter.next().is_some() {
                    return Err(serde::de::Error::invalid_value(
                        serde::de::Unexpected::Map,
                        &"map with a single key",
                    ));
                }
                (variant, Some(value))
            }
            Value::String(variant) => (variant, None),
            other => {
                return Err(serde::de::Error::invalid_type(
                    other.unexpected(),
                    &"string or map",
                ));
            }
        };

        visitor.visit_enum(EnumDeserializer { variant, value })
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct seq tuple
        tuple_struct map struct identifier ignored_any
    }
}

fn visit_decimal<'de, V>(mut d: Decimal, visitor: V) -> Result<V::Value, Error>
where
    V: Visitor<'de>,
{
    d.normalize_assign();
    if d.scale() == 0 {
        if d.is_sign_negative() {
            if let Some(i) = d.to_i64() {
                return visitor.visit_i64(i);
            }
        } else if let Some(u) = d.to_u64() {
            return visitor.visit_u64(u);
        }
    }

    // this is lossy
    if let Some(f) = d.to_f64() {
        return visitor.visit_f64(f);
    }

    // I think to_f64 never fails, so this might be unreachable
    visitor.visit_string(d.to_string())
}

impl Value {
    pub fn unexpected(&self) -> serde::de::Unexpected {
        use serde::de::Unexpected;
        match self {
            Value::Null => Unexpected::Unit,
            Value::String(s) => Unexpected::Str(s),
            Value::Bool(b) => Unexpected::Bool(*b),
            Value::I8(i) => Unexpected::Signed(*i as i64),
            Value::I16(i) => Unexpected::Signed(*i as i64),
            Value::I32(i) => Unexpected::Signed(*i as i64),
            Value::I64(i) => Unexpected::Signed(*i),
            Value::U8(u) => Unexpected::Unsigned(*u as u64),
            Value::U16(u) => Unexpected::Unsigned(*u as u64),
            Value::U32(u) => Unexpected::Unsigned(*u as u64),
            Value::U64(u) => Unexpected::Unsigned(*u),
            Value::F32(f) => Unexpected::Float(*f as f64),
            Value::F64(f) => Unexpected::Float(*f),
            Value::Decimal(_) => Unexpected::Other("decimal"),
            Value::U128(_) => Unexpected::Other("u128"),
            Value::Pubkey(_) => Unexpected::Other("pubkey"),
            Value::Keypair(_) => Unexpected::Other("keypair"),
            Value::Signature(_) => Unexpected::Other("signature"),
            Value::Array(_) => Unexpected::Seq,
            Value::Map(_) => Unexpected::Map,
        }
    }
}

struct EnumDeserializer {
    variant: String,
    value: Option<Value>,
}

impl<'de> serde::de::EnumAccess<'de> for EnumDeserializer {
    type Error = Error;
    type Variant = VariantDeserializer;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, VariantDeserializer), Error>
    where
        V: DeserializeSeed<'de>,
    {
        let variant = self.variant.into_deserializer();
        let visitor = VariantDeserializer { value: self.value };
        seed.deserialize(variant).map(|v| (v, visitor))
    }
}

struct VariantDeserializer {
    value: Option<Value>,
}

impl<'de> serde::de::VariantAccess<'de> for VariantDeserializer {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Error> {
        match self.value {
            Some(value) => serde::Deserialize::deserialize(value),
            None => Ok(()),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.value {
            Some(value) => seed.deserialize(value),
            None => Err(serde::de::Error::invalid_type(
                serde::de::Unexpected::UnitVariant,
                &"newtype variant",
            )),
        }
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Some(Value::Array(v)) => {
                if v.is_empty() {
                    visitor.visit_unit()
                } else {
                    visit_array(v, visitor)
                }
            }
            Some(other) => Err(serde::de::Error::invalid_type(
                other.unexpected(),
                &"tuple variant",
            )),
            None => Err(serde::de::Error::invalid_type(
                serde::de::Unexpected::UnitVariant,
                &"tuple variant",
            )),
        }
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Some(Value::Map(v)) => visit_map(v, visitor),
            Some(other) => Err(serde::de::Error::invalid_type(
                other.unexpected(),
                &"struct variant",
            )),
            None => Err(serde::de::Error::invalid_type(
                serde::de::Unexpected::UnitVariant,
                &"struct variant",
            )),
        }
    }
}

fn visit_array<'de, V>(array: Vec<Value>, visitor: V) -> Result<V::Value, Error>
where
    V: Visitor<'de>,
{
    let len = array.len();
    let mut deserializer = SeqDeserializer::new(array);
    let seq = visitor
        .visit_seq(&mut deserializer)
        .map_err(Error::custom)?;
    let remaining = deserializer.iter.len();
    if remaining == 0 {
        Ok(seq)
    } else {
        Err(serde::de::Error::invalid_length(
            len,
            &"fewer elements in array",
        ))
    }
}

struct SeqDeserializer {
    iter: std::vec::IntoIter<Value>,
}

impl SeqDeserializer {
    fn new(vec: Vec<Value>) -> Self {
        SeqDeserializer {
            iter: vec.into_iter(),
        }
    }
}

impl<'de> serde::de::SeqAccess<'de> for SeqDeserializer {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(value) => seed.deserialize(value).map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self.iter.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}

impl<'de> serde::de::IntoDeserializer<'de, Error> for Value {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

fn visit_map<'de, V>(object: Map, visitor: V) -> Result<V::Value, Error>
where
    V: Visitor<'de>,
{
    let len = object.len();
    let mut deserializer = MapDeserializer::new(object);
    let map = visitor.visit_map(&mut deserializer)?;
    let remaining = deserializer.iter.len();
    if remaining == 0 {
        Ok(map)
    } else {
        Err(serde::de::Error::invalid_length(
            len,
            &"fewer elements in map",
        ))
    }
}

struct MapDeserializer {
    iter: <Map as IntoIterator>::IntoIter,
    value: Option<Value>,
}

impl MapDeserializer {
    fn new(map: Map) -> Self {
        MapDeserializer {
            iter: map.into_iter(),
            value: None,
        }
    }
}

impl<'de> serde::de::MapAccess<'de> for MapDeserializer {
    type Error = Error;

    fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((key, value)) => {
                self.value = Some(value);
                let key_de = MapKeyDeserializer {
                    key: Cow::Owned(key),
                };
                seed.deserialize(key_de).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value, Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.value.take() {
            Some(value) => seed.deserialize(value),
            None => Err(serde::de::Error::custom("value is missing")),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self.iter.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}

struct MapKeyDeserializer<'de> {
    key: Cow<'de, str>,
}

macro_rules! deserialize_integer_key {
    ($method:ident => $visit:ident) => {
        fn $method<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: Visitor<'de>,
        {
            match (self.key.parse(), self.key) {
                (Ok(integer), _) => visitor.$visit(integer),
                (Err(_), Cow::Borrowed(s)) => visitor.visit_borrowed_str(s),
                (Err(_), Cow::Owned(s)) => visitor.visit_string(s),
            }
        }
    };
}

impl<'de> serde::Deserializer<'de> for MapKeyDeserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        BorrowedCowStrDeserializer::new(self.key).deserialize_any(visitor)
    }

    deserialize_integer_key!(deserialize_i8 => visit_i8);
    deserialize_integer_key!(deserialize_i16 => visit_i16);
    deserialize_integer_key!(deserialize_i32 => visit_i32);
    deserialize_integer_key!(deserialize_i64 => visit_i64);
    deserialize_integer_key!(deserialize_u8 => visit_u8);
    deserialize_integer_key!(deserialize_u16 => visit_u16);
    deserialize_integer_key!(deserialize_u32 => visit_u32);
    deserialize_integer_key!(deserialize_u64 => visit_u64);

    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        // Map keys cannot be null.
        visitor.visit_some(self)
    }

    #[inline]
    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.key
            .into_deserializer()
            .deserialize_enum(name, variants, visitor)
    }

    serde::forward_to_deserialize_any! {
        bool f32 f64 char str string bytes byte_buf unit unit_struct seq tuple
        tuple_struct map struct identifier ignored_any
    }
}

struct BorrowedCowStrDeserializer<'de> {
    value: Cow<'de, str>,
}

impl<'de> BorrowedCowStrDeserializer<'de> {
    fn new(value: Cow<'de, str>) -> Self {
        BorrowedCowStrDeserializer { value }
    }
}

impl<'de> serde::de::Deserializer<'de> for BorrowedCowStrDeserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Cow::Borrowed(string) => visitor.visit_borrowed_str(string),
            Cow::Owned(string) => visitor.visit_string(string),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_enum(self)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct identifier ignored_any
    }
}

impl<'de> serde::de::EnumAccess<'de> for BorrowedCowStrDeserializer<'de> {
    type Error = Error;
    type Variant = UnitOnly;

    fn variant_seed<T>(self, seed: T) -> Result<(T::Value, Self::Variant), Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        let value = seed.deserialize(self)?;
        Ok((value, UnitOnly))
    }
}

struct UnitOnly;

impl<'de> serde::de::VariantAccess<'de> for UnitOnly {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, _seed: T) -> Result<T::Value, Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        Err(serde::de::Error::invalid_type(
            serde::de::Unexpected::UnitVariant,
            &"newtype variant",
        ))
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(serde::de::Error::invalid_type(
            serde::de::Unexpected::UnitVariant,
            &"tuple variant",
        ))
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(serde::de::Error::invalid_type(
            serde::de::Unexpected::UnitVariant,
            &"struct variant",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use serde::de::DeserializeOwned;
    use serde::Deserialize;
    use std::collections::{HashMap, HashSet};

    fn de<T: DeserializeOwned>(v: Value) -> T {
        T::deserialize(v).unwrap()
    }

    #[test]
    fn test_primitive() {
        assert_eq!(de::<u8>(Value::U8(0)), 0u8);
        assert_eq!(de::<u8>(Value::U16(0)), 0u8);
        assert_eq!(de::<u8>(Value::U32(0)), 0u8);
        assert_eq!(de::<u8>(Value::U64(0)), 0u8);
        assert_eq!(de::<u8>(Value::U64(0)), 0u8);
        assert_eq!(de::<i8>(Value::I8(0)), 0i8);
        assert_eq!(de::<i8>(Value::I16(0)), 0i8);
        assert_eq!(de::<i8>(Value::I32(0)), 0i8);
        assert_eq!(de::<i8>(Value::I64(0)), 0i8);
        assert_eq!(de::<f32>(Value::F32(0.0)), 0f32);
        assert_eq!(de::<f32>(Value::F64(0.0)), 0f32);
        assert!(!de::<bool>(Value::Bool(false)));
        assert_eq!(de::<String>(Value::String("abc".to_owned())), "abc");
        assert_eq!(de::<f32>(Value::I32(0)), 0f32);
    }

    #[test]
    fn test_option() {
        assert_eq!(de::<Option<u32>>(Value::U32(0)), Some(0));
        assert_eq!(de::<Option<()>>(Value::Null), None);
        assert_eq!(de::<Option<Option<u32>>>(Value::U32(0)), Some(Some(0)));
    }

    #[test]
    fn test_array() {
        assert_eq!(
            de::<Vec<u32>>(Value::Array([Value::U32(0), Value::U8(1)].to_vec())),
            vec![0, 1],
        );

        assert_eq!(
            de::<(u32, f32, Option<u64>, (i32, i32), Vec<String>)>(Value::Array(
                [
                    Value::U32(0),
                    Value::F32(0.1),
                    Value::Null,
                    Value::Array([Value::I32(1), Value::I32(2),].to_vec()),
                    Value::Array([Value::String("hello".to_owned())].to_vec()),
                ]
                .to_vec()
            )),
            (0u32, 0.1f32, None, (1, 2), ["hello".to_owned()].to_vec()),
        );

        assert_eq!(
            de::<HashSet<u32>>(Value::Array([Value::U32(0), Value::U8(1)].to_vec())),
            HashSet::from([0, 1]),
        );
    }

    #[test]
    fn test_wrapper_struct() {
        #[derive(Deserialize, Debug, PartialEq)]
        struct Unit;
        assert_eq!(de::<Unit>(Value::Null), Unit);

        #[derive(Deserialize, Debug, PartialEq)]
        struct Unit1();
        assert_eq!(de::<Unit1>(Value::Array(Vec::new())), Unit1());

        #[derive(Deserialize, Debug, PartialEq)]
        struct NewTypeStruct(i64);
        assert_eq!(de::<NewTypeStruct>(Value::I64(0)), NewTypeStruct(0));

        #[derive(Deserialize, Debug, PartialEq)]
        struct NewTypeStructTuple((i32,));
        assert_eq!(
            de::<NewTypeStructTuple>(Value::Array([Value::I32(0)].to_vec())),
            NewTypeStructTuple((0,))
        );

        #[derive(Deserialize, Debug, PartialEq)]
        struct TupleStruct(i32, String, (i32, i32), (), ((),));
        assert_eq!(
            de::<TupleStruct>(Value::Array(
                [
                    Value::I32(0),
                    Value::String("hello".to_owned()),
                    Value::Array([Value::I32(1), Value::I32(2)].to_vec()),
                    Value::Null,
                    Value::Array([Value::Null].to_vec()),
                ]
                .to_vec()
            )),
            TupleStruct(0, "hello".to_owned(), (1, 2), (), ((),))
        );
    }

    fn bool_true() -> bool {
        true
    }

    fn some_3() -> Option<u32> {
        Some(3)
    }

    #[test]
    fn test_map() {
        assert_eq!(
            de::<HashMap<i32, i32>>(Value::Map(Map::from([
                ("1".to_owned(), Value::I32(2)),
                ("3".to_owned(), Value::I32(4))
            ]))),
            HashMap::<i32, i32>::from([(1, 2), (3, 4)])
        );

        #[derive(Deserialize, Debug, PartialEq)]
        struct Struct {
            x: i32,
            #[serde(default = "bool_true")]
            b0: bool,
            #[serde(rename = "bb")]
            b1: bool,
            #[serde(flatten)]
            flat: Flat,
        }
        #[derive(Deserialize, Debug, PartialEq)]
        struct Flat {
            k: String,
            #[serde(default = "bool_true")]
            b1: bool,
            #[serde(default = "some_3")]
            opt: Option<u32>,
        }
        assert_eq!(
            de::<Struct>(Value::Map(Map::from([
                ("x".to_owned(), Value::I32(1)),
                ("bb".to_owned(), Value::Bool(false)),
                ("k".to_owned(), Value::String("hello".to_owned())),
            ]))),
            Struct {
                x: 1,
                b0: true,
                b1: false,
                flat: Flat {
                    k: "hello".to_owned(),
                    b1: true,
                    opt: Some(3),
                },
            }
        );
    }

    #[test]
    fn test_enum() {
        #[derive(Deserialize, PartialEq, Debug)]
        enum Enum {
            Var1,
            Var2,
            #[serde(rename = "hello")]
            Var3,
        }
        assert_eq!(de::<Enum>(Value::String("Var1".to_owned())), Enum::Var1);
        assert_eq!(de::<Enum>(Value::String("Var2".to_owned())), Enum::Var2);
        assert_eq!(de::<Enum>(Value::String("hello".to_owned())), Enum::Var3);

        #[derive(Deserialize, PartialEq, Debug)]
        #[serde(untagged)]
        enum Enum1 {
            A { a: u32 },
            BC { b: Option<u32>, c: Option<u32> },
        }
        assert_eq!(
            de::<Enum1>(Value::Map(Map::from([("a".to_owned(), Value::U32(0))]))),
            Enum1::A { a: 0 }
        );
        assert_eq!(
            de::<Enum1>(Value::Map(Map::new())),
            Enum1::BC { b: None, c: None }
        );

        #[derive(Deserialize, PartialEq, Debug)]
        enum Enum2 {
            A { a: u32 },
            BC { b: Option<u32>, c: Option<u32> },
            D,
            E(f32),
        }
        assert_eq!(
            de::<Enum2>(Value::Map(Map::from([(
                "A".to_owned(),
                Value::Map(Map::from([("a".to_owned(), Value::U32(0))]))
            )]))),
            Enum2::A { a: 0 }
        );
        assert_eq!(
            de::<Enum2>(Value::Map(Map::from([(
                "BC".to_owned(),
                Value::Map(Map::new())
            )]))),
            Enum2::BC { b: None, c: None }
        );
        assert_eq!(
            de::<Enum2>(Value::Map(Map::from([("D".to_owned(), Value::Null)]))),
            Enum2::D,
        );
        assert_eq!(
            de::<Enum2>(Value::Map(Map::from([("E".to_owned(), Value::F32(0.0),)]))),
            Enum2::E(0.0),
        );
    }

    #[test]
    fn test_decimal() {
        assert_eq!(de::<u32>(Value::Decimal(dec!(100.0))), 100);
        assert_eq!(de::<f32>(Value::Decimal(dec!(100))), 100.0);
        assert_eq!(de::<f64>(Value::Decimal(dec!(1999.1234))), 1999.1234);
        assert_eq!(
            de::<f64>(Value::Decimal(Decimal::MAX)),
            7.922816251426434e28
        );
        assert_eq!(de::<u64>(Value::Decimal(Decimal::from(u64::MAX))), u64::MAX);
    }

    #[test]
    fn test_custom_types() {}
}
