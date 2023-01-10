use crate::{value_type::Variant, Value};
use serde::de::VariantAccess;
use std::borrow::Cow;

pub mod iter_ser;

#[derive(Debug)]
pub struct JsonRepr<'a>(std::borrow::Cow<'a, Value>);

impl<'a> JsonRepr<'a> {
    pub fn new(value: &Value) -> JsonRepr<'_> {
        JsonRepr(Cow::Borrowed(value))
    }

    pub fn into_value(self) -> Value {
        match self.0 {
            Cow::Borrowed(value) => value.clone(),
            Cow::Owned(value) => value,
        }
    }
}

impl<'a> AsRef<Value> for JsonRepr<'a> {
    fn as_ref(&self) -> &Value {
        &self.0
    }
}

impl<'a> serde::Serialize for JsonRepr<'a> {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        const NAME: &str = "JsonRepr";

        let value = self.as_ref();
        let (i, k) = value.kind().variant();
        match value {
            Value::Null => s.serialize_newtype_variant(NAME, i, k, &0),
            Value::String(v) => s.serialize_newtype_variant(NAME, i, k, v),
            Value::Bool(v) => s.serialize_newtype_variant(NAME, i, k, v),
            Value::U64(v) => {
                s.serialize_newtype_variant(NAME, i, k, itoa::Buffer::new().format(*v))
            }
            Value::I64(v) => {
                s.serialize_newtype_variant(NAME, i, k, itoa::Buffer::new().format(*v))
            }
            Value::F64(v) => s.serialize_newtype_variant(NAME, i, k, &v.to_string()),
            Value::Decimal(v) => {
                // TODO: no alloc impl
                s.serialize_newtype_variant(NAME, i, k, &v.to_string())
            }
            Value::I128(v) => {
                s.serialize_newtype_variant(NAME, i, k, itoa::Buffer::new().format(*v))
            }
            Value::U128(v) => {
                s.serialize_newtype_variant(NAME, i, k, itoa::Buffer::new().format(*v))
            }
            Value::B32(v) => {
                s.serialize_newtype_variant(NAME, i, k, &bs58::encode(v).into_string())
            }
            Value::B64(v) => {
                s.serialize_newtype_variant(NAME, i, k, &bs58::encode(v).into_string())
            }
            Value::Bytes(v) => s.serialize_newtype_variant(NAME, i, k, &base64::encode(v)),
            Value::Array(v) => s.serialize_newtype_variant(
                NAME,
                i,
                k,
                &iter_ser::Array::new(v.iter().map(JsonRepr::new)),
            ),
            Value::Map(v) => s.serialize_newtype_variant(
                NAME,
                i,
                k,
                &iter_ser::Map::new(v.iter().map(|(k, v)| (k, JsonRepr::new(v)))),
            ),
        }
    }
}

struct EnumVisitor;

impl<'de> serde::de::Visitor<'de> for EnumVisitor {
    type Value = Value;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("any valid value")
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::EnumAccess<'de>,
    {
        let (ty, a) = data.variant::<Variant>()?;
        match ty {
            Variant::Null => {
                let num = a.newtype_variant::<u64>()?;
                if num != 0 {
                    return Err(serde::de::Error::invalid_value(
                        serde::de::Unexpected::Unsigned(num),
                        &"0",
                    ));
                }
                Ok(Value::Null)
            }
            Variant::String => Ok(Value::String(a.newtype_variant()?)),
            Variant::Bool => Ok(Value::Bool(a.newtype_variant()?)),
            Variant::U64 => Ok(Value::U64(number_from_str(a)?)),
            Variant::I64 => Ok(Value::I64(number_from_str(a)?)),
            Variant::F64 => Ok(Value::F64(number_from_str(a)?)),
            Variant::Decimal => Ok(Value::Decimal(number_from_str(a)?)),
            Variant::I128 => Ok(Value::I128(number_from_str(a)?)),
            Variant::U128 => Ok(Value::U128(number_from_str(a)?)),
            Variant::B32 => Ok(Value::B32(b58_str(a)?)),
            Variant::B64 => Ok(Value::B64(b58_str(a)?)),
            Variant::Bytes => Ok(Value::Bytes(b64_str(a)?)),
            Variant::Array => Ok(Value::Array(a.newtype_variant::<JsonArray>()?.0)),
            Variant::Map => Ok(Value::Map(a.newtype_variant::<JsonMap>()?.0)),
        }
    }
}

struct MapVisitor;

impl<'de> serde::de::Visitor<'de> for MapVisitor {
    type Value = crate::Map;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("map")
    }

    fn visit_map<A>(self, mut a: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut map = crate::Map::new();
        if let Some(len) = a.size_hint() {
            map.reserve(len);
        }
        while let Some((k, v)) = a.next_entry::<crate::Key, JsonRepr<'static>>()? {
            map.insert(k, v.into_value());
        }
        Ok(map)
    }
}

struct JsonMap(crate::Map);

impl<'de> serde::Deserialize<'de> for JsonMap {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(JsonMap(d.deserialize_map(MapVisitor)?))
    }
}

struct ArrayVisitor;

impl<'de> serde::de::Visitor<'de> for ArrayVisitor {
    type Value = Vec<Value>;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("array")
    }

    fn visit_seq<A>(self, mut a: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut vec = Vec::new();
        if let Some(len) = a.size_hint() {
            vec.reserve(len);
        }
        while let Some(v) = a.next_element::<JsonRepr<'static>>()? {
            vec.push(v.into_value());
        }
        Ok(vec)
    }
}

struct JsonArray(Vec<Value>);

impl<'de> serde::Deserialize<'de> for JsonArray {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(JsonArray(d.deserialize_seq(ArrayVisitor)?))
    }
}

fn number_from_str<'de, A, T>(a: A) -> Result<T, A::Error>
where
    A: VariantAccess<'de>,
    T: std::str::FromStr,
{
    let s = a.newtype_variant::<Cow<'_, str>>()?;
    s.parse::<T>()
        .map_err(|_| serde::de::Error::custom(format!("invalid number: {}", s)))
}

fn b58_str<'de, A, const N: usize>(a: A) -> Result<[u8; N], A::Error>
where
    A: VariantAccess<'de>,
{
    let mut buf = [0u8; N];
    let s = a.newtype_variant::<Cow<'_, str>>()?;
    let size = bs58::decode(&*s)
        .into(&mut buf)
        .map_err(|_| serde::de::Error::custom("invalid base58"))?;
    if size != N {
        return Err(serde::de::Error::invalid_length(
            size,
            &itoa::Buffer::new().format(N),
        ));
    }
    Ok(buf)
}

fn b64_str<'de, A>(a: A) -> Result<bytes::Bytes, A::Error>
where
    A: VariantAccess<'de>,
{
    let s = a.newtype_variant::<Cow<'_, str>>()?;
    base64::decode(&*s)
        .map_err(|_| serde::de::Error::custom("invalid base64"))
        .map(Into::into)
}

impl<'a, 'de> serde::Deserialize<'de> for JsonRepr<'a> {
    /// Turn any `Deserializer` into `Value`, intended to be used
    /// with `Value as Deserializer`.
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = d.deserialize_enum(crate::TOKEN, crate::value_type::keys::ALL, EnumVisitor)?;
        Ok(JsonRepr(std::borrow::Cow::Owned(value)))
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::*;
    use crate::map;

    #[test]
    fn test_json() {
        fn t(v: Value, s: &str) {
            assert_eq!(s, serde_json::to_string(&JsonRepr::new(&v)).unwrap());
            assert_eq!(
                &v,
                serde_json::from_str::<JsonRepr<'_>>(s).unwrap().as_ref()
            );
        }
        t(Value::Null, r#"{"N":0}"#);
        t(Value::String("hello".to_owned()), r#"{"S":"hello"}"#);
        t(Value::U64(0), r#"{"U":"0"}"#);
        t(Value::I64(-1), r#"{"I":"-1"}"#);
        t(
            Value::U128(u128::MAX),
            r#"{"U1":"340282366920938463463374607431768211455"}"#,
        );
        t(
            Value::I128(i128::MIN),
            r#"{"I1":"-170141183460469231731687303715884105728"}"#,
        );
        t(Value::Bool(true), r#"{"B":true}"#);
        t(
            Value::Decimal(dec!(3.1415926535897932384626433833)),
            r#"{"D":"3.1415926535897932384626433833"}"#,
        );
        t(
            Value::Map(map! {
                "foo" => 1i64,
            }),
            r#"{"M":{"foo":{"I":"1"}}}"#,
        );
        t(
            Value::Array(vec![1i64.into(), "hello".into()]),
            r#"{"A":[{"I":"1"},{"S":"hello"}]}"#,
        );
        t(
            Value::B32(
                bs58::decode("5sNRWMrT2P3KULzW3faaktCB3k2eqHow2GBJtcsCPcg7")
                    .into_vec()
                    .unwrap()
                    .try_into()
                    .unwrap(),
            ),
            r#"{"B3":"5sNRWMrT2P3KULzW3faaktCB3k2eqHow2GBJtcsCPcg7"}"#,
        );
        t(
            Value::B64(
                bs58::decode("3PvNxykqBz1BzBaq2AMU4Sa3CPJGnSC9JXkyzXe33m6W7Sj4MMgsZet6YxUQdPx1fEFU79QWm6RpPRVJAyeqiNsR")
                    .into_vec()
                    .unwrap()
                    .try_into()
                    .unwrap(),
            ),
            r#"{"B6":"3PvNxykqBz1BzBaq2AMU4Sa3CPJGnSC9JXkyzXe33m6W7Sj4MMgsZet6YxUQdPx1fEFU79QWm6RpPRVJAyeqiNsR"}"#,
        );
        t(
            Value::Bytes(bytes::Bytes::from_static(&[
                104, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100,
            ])),
            r#"{"BY":"aGVsbG8gd29ybGQ="}"#,
        );
    }
}
