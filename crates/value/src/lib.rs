use rust_decimal::Decimal;
use thiserror::Error as ThisError;

pub(crate) mod value_type;

pub(crate) const TOKEN: &str = "$V";

pub mod crud;
pub mod de;
pub mod json_repr;
pub mod macros;
pub mod ser;

// custom serialize and deserialize modules
pub mod decimal;
#[cfg(feature = "solana")]
pub mod keypair;
#[cfg(feature = "solana")]
pub mod pubkey;
#[cfg(feature = "solana")]
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

// allow for switching HashMap implementation
pub type HashMap<K, V> = indexmap::IndexMap<K, V>;

// could use smartstring?
pub type Key = String;

pub type Map = self::HashMap<Key, Value>;

#[derive(Clone, PartialEq)]
pub enum Value {
    Null,
    String(String),
    Bool(bool),
    U64(u64),
    I64(i64),
    F64(f64),
    Decimal(Decimal),
    I128(i128),
    U128(u128),
    B32([u8; 32]),
    B64([u8; 64]),
    Bytes(bytes::Bytes),
    Array(Vec<Self>),
    Map(Map),
}

impl Value {
    pub fn new_keypair_bs58(s: &str) -> Result<Self, Error> {
        // and Ed25519 keypair
        const KEYPAIR_LENGTH: usize = 64;
        let mut buf = [0u8; KEYPAIR_LENGTH];
        let size = bs58::decode(s).into(&mut buf)?;
        if size != KEYPAIR_LENGTH {
            return Err(Error::InvalidLenght {
                need: KEYPAIR_LENGTH,
                got: size,
            });
        }

        Ok(Value::B64(buf))
    }
}

impl Default for Value {
    fn default() -> Self {
        Value::Null
    }
}

impl From<serde_json::Value> for Value {
    fn from(value: serde_json::Value) -> Self {
        match value {
            serde_json::Value::Null => Value::Null,
            serde_json::Value::Bool(b) => Value::Bool(b),
            serde_json::Value::Number(n) => {
                if let Some(u) = n.as_u64() {
                    Value::U64(u)
                } else if let Some(i) = n.as_i64() {
                    if i < 0 {
                        Value::I64(i)
                    } else {
                        Value::U64(i as u64)
                    }
                } else {
                    let s = n.to_string();
                    if let Ok(u) = s.parse::<u128>() {
                        Value::U128(u)
                    } else if let Ok(i) = s.parse::<i128>() {
                        Value::I128(i)
                    } else if let Ok(d) = s.parse::<Decimal>() {
                        Value::Decimal(d)
                    } else if let Ok(d) = Decimal::from_scientific(&s) {
                        Value::Decimal(d)
                    } else if let Ok(f) = s.parse::<f64>() {
                        Value::F64(f)
                    } else {
                        // unlikely to happen
                        // if happen, probably a bug in serde_json
                        Value::String(s)
                    }
                }
            }
            serde_json::Value::String(s) => Value::String(s),
            serde_json::Value::Array(vec) => {
                Value::Array(vec.into_iter().map(Value::from).collect())
            }
            serde_json::Value::Object(map) => {
                Value::Map(map.into_iter().map(|(k, v)| (k, Value::from(v))).collect())
            }
        }
    }
}

impl From<String> for Value {
    fn from(x: String) -> Self {
        Self::String(x)
    }
}

impl From<&str> for Value {
    fn from(x: &str) -> Self {
        Self::String(x.to_owned())
    }
}

impl From<bool> for Value {
    fn from(x: bool) -> Self {
        Self::Bool(x)
    }
}

impl From<u8> for Value {
    fn from(x: u8) -> Self {
        Self::U64(x as u64)
    }
}

impl From<u16> for Value {
    fn from(x: u16) -> Self {
        Self::U64(x as u64)
    }
}

impl From<u32> for Value {
    fn from(x: u32) -> Self {
        Self::U64(x as u64)
    }
}

impl From<u64> for Value {
    fn from(x: u64) -> Self {
        Self::U64(x)
    }
}

impl From<u128> for Value {
    fn from(x: u128) -> Self {
        Self::U128(x)
    }
}

impl From<i8> for Value {
    fn from(x: i8) -> Self {
        Self::I64(x as i64)
    }
}

impl From<i16> for Value {
    fn from(x: i16) -> Self {
        Self::I64(x as i64)
    }
}

impl From<i32> for Value {
    fn from(x: i32) -> Self {
        Self::I64(x as i64)
    }
}

impl From<i64> for Value {
    fn from(x: i64) -> Self {
        Self::I64(x)
    }
}

impl From<i128> for Value {
    fn from(x: i128) -> Self {
        Self::I128(x)
    }
}

impl From<f32> for Value {
    fn from(x: f32) -> Self {
        Self::F64(x as f64)
    }
}

impl From<f64> for Value {
    fn from(x: f64) -> Self {
        Self::F64(x)
    }
}

impl From<[u8; 32]> for Value {
    fn from(x: [u8; 32]) -> Self {
        Self::B32(x)
    }
}

impl From<[u8; 64]> for Value {
    fn from(x: [u8; 64]) -> Self {
        Self::B64(x)
    }
}

#[cfg(feature = "solana")]
impl From<solana_sdk::pubkey::Pubkey> for Value {
    fn from(x: solana_sdk::pubkey::Pubkey) -> Self {
        Self::B32(x.to_bytes())
    }
}

#[cfg(feature = "solana")]
impl From<solana_sdk::signer::keypair::Keypair> for Value {
    fn from(x: solana_sdk::signer::keypair::Keypair) -> Self {
        Self::B64(x.to_bytes())
    }
}

#[cfg(feature = "solana")]
impl From<solana_sdk::signature::Signature> for Value {
    fn from(x: solana_sdk::signature::Signature) -> Self {
        Self::B64(x.into())
    }
}

impl From<bytes::Bytes> for Value {
    fn from(x: bytes::Bytes) -> Self {
        match x.len() {
            32 => Self::B32(<_>::try_from(&*x).unwrap()),
            64 => Self::B64(<_>::try_from(&*x).unwrap()),
            _ => Self::Bytes(x),
        }
    }
}

impl From<&[u8]> for Value {
    fn from(x: &[u8]) -> Self {
        match x.len() {
            32 => Self::B32(<_>::try_from(x).unwrap()),
            64 => Self::B64(<_>::try_from(x).unwrap()),
            _ => Self::Bytes(bytes::Bytes::copy_from_slice(x)),
        }
    }
}

impl From<Vec<u8>> for Value {
    fn from(x: Vec<u8>) -> Self {
        match x.len() {
            32 => Self::B32(<_>::try_from(&*x).unwrap()),
            64 => Self::B64(<_>::try_from(&*x).unwrap()),
            _ => Self::Bytes(x.into()),
        }
    }
}

impl From<Vec<Value>> for Value {
    fn from(x: Vec<Value>) -> Self {
        Self::Array(x)
    }
}

impl From<Map> for Value {
    fn from(x: Map) -> Self {
        Self::Map(x)
    }
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Null => f.debug_tuple("Null").finish(),
            Value::String(x) => f.debug_tuple("String").field(x).finish(),
            Value::Bool(x) => f.debug_tuple("Bool").field(x).finish(),
            Value::I64(x) => f.debug_tuple("I64").field(x).finish(),
            Value::U64(x) => f.debug_tuple("U64").field(x).finish(),
            Value::F64(x) => f.debug_tuple("F64").field(x).finish(),
            Value::Decimal(x) => f.debug_tuple("Decimal").field(x).finish(),
            Value::I128(x) => f.debug_tuple("I128").field(x).finish(),
            Value::U128(x) => f.debug_tuple("U128").field(x).finish(),
            Value::Array(x) => f.debug_tuple("Array").field(x).finish(),
            Value::Map(x) => f.debug_tuple("Map").field(x).finish(),
            Value::Bytes(x) => f.debug_tuple("Bytes").field(&x.len()).finish(),
            Value::B32(x) => f
                .debug_tuple("B32")
                .field(&bs58::encode(x).into_string())
                .finish(),
            Value::B64(x) => f
                .debug_tuple("B64")
                .field(&bs58::encode(x).into_string())
                .finish(),
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
        D: serde::Deserializer<'de>,
    {
        d.deserialize_any(self.0).map(Some)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /*
    #[test]
    fn de_pubkey() {
        from_value::<Pubkey>(Value::Pubkey(Pubkey::new(&[0; 32]))).unwrap();
    }

    #[test]
    fn serde() {
        #[derive(Deserialize, Serialize, Debug)]
        struct Foo {
            key: Value,
            pk: Pubkey,
        }

        let value = Value::Map(map! {
            "key" => Value::Map(map! {
                "a" => Value::I8(0),
                "b" => Value::Pubkey(Pubkey::new(&[1; 32])),
            }),
            "pk" => Value::Pubkey(Pubkey::new(&[0; 32])),
        });
        let v = to_value(&from_value::<Foo>(value.clone()).unwrap()).unwrap();
        assert_eq!(v, value);
    }
    */

    #[test]
    fn test_solana_instruction() {
        use solana_sdk::instruction::{AccountMeta, Instruction};
        use solana_sdk::pubkey;

        let i = Instruction::new_with_bytes(
            pubkey!("ESxeViFP4r7THzVx9hJDkhj4HrNGSjJSFRPbGaAb97hN"),
            &[100; 1024],
            vec![AccountMeta {
                pubkey: pubkey!("ESxeViFP4r7THzVx9hJDkhj4HrNGSjJSFRPbGaAb97hN"),
                is_signer: true,
                is_writable: false,
            }],
        );

        let v = to_value(&i).unwrap();
        dbg!(&v);

        let i1: Instruction = from_value(v).unwrap();

        assert_eq!(i, i1);
    }
}
