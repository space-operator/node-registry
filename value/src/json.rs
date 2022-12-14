use serde::{
    de::{MapAccess, SeqAccess, Visitor},
    ser::{SerializeMap, SerializeSeq},
    Deserialize, Deserializer, Serialize, Serializer,
};

use crate::{Map, Value};

impl Serialize for Value {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Value::Null => s.serialize_none(),
            Value::String(x) => s.serialize_str(x),
            Value::Bool(x) => s.serialize_bool(*x),
            Value::I8(x) => s.serialize_i8(*x),
            Value::I16(x) => s.serialize_i16(*x),
            Value::I32(x) => s.serialize_i32(*x),
            Value::I64(x) => s.serialize_i64(*x),
            Value::U8(x) => s.serialize_u8(*x),
            Value::U16(x) => s.serialize_u16(*x),
            Value::U32(x) => s.serialize_u32(*x),
            Value::U64(x) => s.serialize_u64(*x),
            Value::F32(x) => s.serialize_f32(*x),
            Value::F64(x) => s.serialize_f64(*x),
            // rust_decimal's function doesn't allocate
            Value::Decimal(x) => rust_decimal::serde::str::serialize(x, s),
            Value::U128(x) => s.serialize_u128(*x),
            // TODO: implement with no-allocation, is it worth doing?
            Value::Pubkey(x) => s.serialize_str(&x.to_string()),
            Value::Keypair(x) => s.serialize_str(&solana_sdk::bs58::encode(x).into_string()),
            Value::Signature(x) => s.serialize_str(&x.to_string()),
            Value::Array(x) => {
                let mut seq = s.serialize_seq(Some(x.len()))?;
                for v in x {
                    seq.serialize_element(v)?;
                }
                seq.end()
            }
            Value::Map(x) => {
                let mut map = s.serialize_map(Some(x.len()))?;
                for (k, v) in x {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
        }
    }
}

struct ValueVisitor;

impl<'de> Visitor<'de> for ValueVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("any valid JSON value")
    }

    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::I8(v))
    }

    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::I16(v))
    }

    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::I32(v))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::I64(v))
    }

    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::U8(v))
    }

    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::U16(v))
    }

    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::U32(v))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::U64(v))
    }

    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::F32(v))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::F64(v))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Null)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Null)
    }

    fn visit_some<D>(self, d: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        Deserialize::deserialize(d)
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::String(v))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::String(v.to_owned()))
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Bool(v))
    }

    fn visit_seq<A>(self, mut a: A) -> Result<Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut vec = Vec::with_capacity(a.size_hint().unwrap_or(0));
        while let Some(value) = a.next_element()? {
            vec.push(value);
        }
        Ok(Value::Array(vec))
    }

    fn visit_map<A>(self, mut a: A) -> Result<Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut map = Map::with_capacity(a.size_hint().unwrap_or(0));
        while let Some((key, value)) = a.next_entry()? {
            map.insert(key, value);
        }
        Ok(Value::Map(map))
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        d.deserialize_any(ValueVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::signer::keypair::Keypair;

    #[test]
    fn test_json_serialize() {
        let result = include_str!("tests/ser.json").trim();

        let array = [
            Value::I8(i8::MIN),
            Value::I16(i16::MIN),
            Value::I32(i32::MIN),
            Value::I64(i64::MIN),
            Value::U8(u8::MAX),
            Value::U16(u16::MAX),
            Value::U32(u32::MAX),
            Value::U64(u64::MAX),
            Value::Null,
            Value::String("some\nstring".to_owned()),
            Value::Bool(true),
            Value::Bool(false),
            Value::Decimal("1.121".parse().unwrap()),
            Value::F64(1.121),
            Value::F32(1.121),
            Value::Pubkey(
                "GQZRKDqVzM4DXGGMEUNdnBD3CC4TTywh3PwgjYPBm8W9"
                    .parse()
                    .unwrap(),
            ),
            Value::Keypair(Keypair::from_base58_string("56Ngo8EY5ZWmYKDZAmKYcUf2y2LZVRSMMnptGp9JtQuSZHyU3Pwhhkmj5YVf89VTQZqrzkabhybWdWwJWCa74aYu").to_bytes()),
            Value::Signature("6pc4LiB8KHAPvbUbkozrTcPL5zXspYBdATv5raNDyVbhiKjrKokLb9o111kxTD5KkPVd7UBSCcFcnWFkrJ82Hu6".parse().unwrap()),
        ];

        let json = serde_json::to_string(&array).unwrap();
        assert_eq!(json, result);

        let json = serde_json::to_string(&Value::Array(array.to_vec())).unwrap();
        assert_eq!(json, result);

        // hashmap is random, can't compare
        serde_json::to_string(&Value::Map(
            array
                .into_iter()
                .enumerate()
                .map(|(i, v)| (i.to_string(), v))
                .collect(),
        ))
        .unwrap();
    }

    #[test]
    fn test_json_deserialize() {
        fn de(s: &str) -> Value {
            serde_json::from_str(s).unwrap()
        }

        // serde_json always deserializes to the largest number type
        assert_eq!(de("12"), Value::U64(12));
        assert_eq!(de("-12"), Value::I64(-12));
        assert_eq!(de("-12.0"), Value::F64(-12.0));

        // cannot "hint" that we are expecting pubkey or decimal
        // it always deserializes to a string
        assert_eq!(
            de(r#""GQZRKDqVzM4DXGGMEUNdnBD3CC4TTywh3PwgjYPBm8W9""#),
            Value::String("GQZRKDqVzM4DXGGMEUNdnBD3CC4TTywh3PwgjYPBm8W9".into())
        );
        assert_eq!(de(r#""100.0000001""#), Value::String("100.0000001".into()));

        assert_eq!(
            de(r#"[null, {"k": 1}, false]"#),
            Value::Array(
                [
                    Value::Null,
                    Value::Map([("k".into(), Value::U64(1))].into()),
                    Value::Bool(false)
                ]
                .to_vec()
            )
        );
    }
}
