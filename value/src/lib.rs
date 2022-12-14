use ed25519_dalek::KEYPAIR_LENGTH;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use thiserror::Error as ThisError;

pub mod crud;
pub mod de;
pub mod json;
pub mod macros;
pub mod ser;

// custom serialize and deserialize modules
pub mod decimal;
pub mod keypair;
pub mod pubkey;
pub mod signature;

pub fn from_value<T>(value: Value) -> Result<T, Error>
where
    T: for<'de> serde::Deserialize<'de>,
{
    T::deserialize(value)
}

pub fn from_map<T>(map: Map) -> Result<T, Error>
where
    T: for<'de> serde::Deserialize<'de>,
{
    T::deserialize(Value::Map(map))
}

pub fn to_value<T>(t: &T) -> Result<Value, Error>
where
    T: serde::Serialize,
{
    t.serialize(ser::Serializer)
}

pub fn to_map<T>(t: &T) -> Result<Map, Error>
where
    T: serde::Serialize,
{
    to_value(t).and_then(|v| {
        if let Value::Map(map) = v {
            Ok(map)
        } else {
            Err(Error::ExpectedMap)
        }
    })
}

pub(crate) mod tokens {
    pub(crate) const DECIMAL: &str = "$$decimal";
    pub(crate) const PUBKEY: &str = "$$pubkey";
    pub(crate) const KEYPAIR: &str = "$$keypair";
    pub(crate) const SIGNATURE: &str = "$$signature";
}

// allow for switching HashMap implementation
pub type HashMap<K, V> = indexmap::IndexMap<K, V>;

// could use smartstring?
pub type Key = String;

pub type Map = self::HashMap<Key, Value>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValueType {
    #[serde(rename = "bool")]
    Bool,
    #[serde(rename = "u8")]
    U8,
    #[serde(rename = "u16")]
    U16,
    #[serde(rename = "u32")]
    U32,
    #[serde(rename = "u64")]
    U64,
    #[serde(rename = "u128")]
    U128,
    #[serde(rename = "i8")]
    I8,
    #[serde(rename = "i16")]
    I16,
    #[serde(rename = "i32")]
    I32,
    #[serde(rename = "i64")]
    I64,
    #[serde(rename = "f32")]
    F32,
    #[serde(rename = "f64")]
    F64,
    #[serde(rename = "pubkey")]
    Pubkey,
    #[serde(rename = "keypair")]
    Keypair,
    #[serde(rename = "signature")]
    Signature,
    #[serde(rename = "string")]
    String,
    #[serde(rename = "array")]
    Array(Box<ValueType>),
    #[serde(rename = "object")]
    Map(HashMap<Key, ValueType>),
    #[serde(rename = "json")]
    Json,
    #[serde(alias = "file")]
    #[serde(rename = "free")]
    Free,
}

impl ValueType {
    pub fn is_complex(&self) -> bool {
        matches!(
            self,
            ValueType::Map(_) | ValueType::Array(_) | ValueType::String
        )
    }

    pub fn is_number(&self) -> bool {
        matches!(
            self,
            ValueType::U8
                | ValueType::U16
                | ValueType::U32
                | ValueType::U64
                | ValueType::U128
                | ValueType::I8
                | ValueType::I16
                | ValueType::I32
                | ValueType::I64
                | ValueType::F32
                | ValueType::F64
        )
    }
}

#[derive(Clone, PartialEq)]
pub enum Value {
    Null,
    String(String),
    Bool(bool),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    Decimal(Decimal),
    U128(u128),
    Pubkey(Pubkey),
    // ed25519_dalek::Keypair is 252 bytes
    Keypair([u8; KEYPAIR_LENGTH]),
    Signature(Signature),
    Array(Vec<Self>),
    Map(Map),
}

impl Value {
    pub fn to_type(&self) -> Option<ValueType> {
        Some(match self {
            Value::String(_) => ValueType::String,
            Value::Bool(_) => ValueType::Bool,
            Value::I8(_) => ValueType::I8,
            Value::I16(_) => ValueType::I16,
            Value::I32(_) => ValueType::I32,
            Value::I64(_) => ValueType::I64,
            Value::U8(_) => ValueType::U8,
            Value::U16(_) => ValueType::U16,
            Value::U32(_) => ValueType::U32,
            Value::U64(_) => ValueType::U64,
            Value::F32(_) => ValueType::F32,
            Value::F64(_) => ValueType::F64,
            Value::U128(_) => ValueType::U128,
            Value::Pubkey(_) => ValueType::Pubkey,
            Value::Keypair(_) => ValueType::Keypair,
            Value::Signature(_) => ValueType::Signature,
            Value::Array(v) => ValueType::Array(Box::new(v.get(0).map(|it| it.to_type())??)),
            Value::Map(v) => ValueType::Map(
                v.iter()
                    .filter_map(|(key, value)| value.to_type().map(|r#type| (key.clone(), r#type)))
                    .collect(),
            ),
            _ => return None,
        })
    }

    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }

    pub fn from_json_value(v: serde_json::Value) -> Self {
        // Value is a superset of serde_json::Value,
        // so this should never fail
        serde_json::from_value(v).expect("should never fail")
    }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::to_value(self).expect("should never fail")
    }

    pub fn new_keypair_bs58(s: &str) -> Result<Self, Error> {
        let mut buf = [0u8; KEYPAIR_LENGTH];
        let size = bs58::decode(s).into(&mut buf)?;
        if size != KEYPAIR_LENGTH {
            return Err(Error::InvalidLenght {
                need: KEYPAIR_LENGTH,
                got: size,
            });
        }

        Ok(Value::Keypair(buf))
    }
}

impl Default for Value {
    fn default() -> Self {
        Value::Null
    }
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Null => f.debug_tuple("Null").finish(),
            Value::String(x) => f.debug_tuple("String").field(x).finish(),
            Value::Bool(x) => f.debug_tuple("Bool").field(x).finish(),
            Value::I8(x) => f.debug_tuple("I8").field(x).finish(),
            Value::I16(x) => f.debug_tuple("I16").field(x).finish(),
            Value::I32(x) => f.debug_tuple("I32").field(x).finish(),
            Value::I64(x) => f.debug_tuple("I64").field(x).finish(),
            Value::U8(x) => f.debug_tuple("U8").field(x).finish(),
            Value::U16(x) => f.debug_tuple("U16").field(x).finish(),
            Value::U32(x) => f.debug_tuple("U32").field(x).finish(),
            Value::U64(x) => f.debug_tuple("U64").field(x).finish(),
            Value::F32(x) => f.debug_tuple("F32").field(x).finish(),
            Value::F64(x) => f.debug_tuple("F64").field(x).finish(),
            Value::Decimal(x) => f.debug_tuple("Decimal").field(x).finish(),
            Value::U128(x) => f.debug_tuple("U128").field(x).finish(),
            Value::Pubkey(x) => f.debug_tuple("Pubkey").field(x).finish(),
            Value::Keypair(x) => f
                .debug_tuple("Keypair")
                .field(&bs58::encode(&x).into_string())
                .finish(),
            Value::Signature(x) => f.debug_tuple("Signature").field(x).finish(),
            Value::Array(x) => f.debug_tuple("Array").field(x).finish(),
            Value::Map(x) => f.debug_tuple("Map").field(x).finish(),
        }
    }
}

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("{0}")]
    Custom(String),
    #[error("key must be a string")]
    KeyMustBeAString,
    #[error("invalid base58: {0}")]
    Bs58Decode(#[from] bs58::decode::Error),
    #[error("need length {need}, got {got}")]
    InvalidLenght { need: usize, got: usize },
    #[error("expected a map")]
    ExpectedMap,
}

impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::Custom(msg.to_string())
    }
}

impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::Custom(msg.to_string())
    }
}

// default implementation of [u8] doesn't call serialize_bytes
pub(crate) struct Bytes<'a>(&'a [u8]);

impl<'a> serde::Serialize for Bytes<'a> {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        s.serialize_bytes(self.0)
    }
}

pub mod default {
    pub fn bool_true() -> bool {
        true
    }
}

pub(crate) struct OptionVisitor<V>(pub(crate) V);

impl<'de, V> serde::de::Visitor<'de> for OptionVisitor<V>
where
    V: serde::de::Visitor<'de>,
{
    type Value = Option<V::Value>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("optional ")?;
        self.0.expecting(formatter)
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(None)
    }

    fn visit_some<D>(self, d: D) -> Result<Self::Value, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        d.deserialize_any(self.0).map(Some)
    }
}
