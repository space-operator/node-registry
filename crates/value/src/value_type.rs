use crate::Value;

#[derive(Clone, Copy, Debug)]
#[repr(u32)]
pub enum ValueType {
    Null = 0,
    String = 1,
    Bool = 2,
    U64 = 3,
    I64 = 4,
    F64 = 5,
    Decimal = 6,
    I128 = 7,
    U128 = 8,
    B32 = 9,
    B64 = 10,
    Bytes = 11,
    Array = 12,
    Map = 13,
}

impl ValueType {
    pub const fn variant(&self) -> (u32, &'static str) {
        let idx = *self as u32;
        (idx, keys::ALL[idx as usize])
    }
}

pub mod keys {
    pub const NULL: &str = "N";
    pub const STRING: &str = "S";
    pub const BOOL: &str = "B";
    pub const U64: &str = "U";
    pub const I64: &str = "I";
    pub const F64: &str = "F";
    pub const DECIMAL: &str = "D";
    pub const I128: &str = "I1";
    pub const U128: &str = "U1";
    pub const B32: &str = "B3";
    pub const B64: &str = "B6";
    pub const BYTES: &str = "BY";
    pub const ARRAY: &str = "A";
    pub const MAP: &str = "M";

    pub const ALL: &[&str] = &[
        NULL, STRING, BOOL, U64, I64, F64, DECIMAL, I128, U128, B32, B64, BYTES, ARRAY, MAP,
    ];
}

struct ValueTypeVisitor;

impl<'de> serde::de::Visitor<'de> for ValueTypeVisitor {
    type Value = ValueType;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("ValueType")
    }

    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(match v {
            0 => ValueType::Null,
            1 => ValueType::String,
            2 => ValueType::Bool,
            3 => ValueType::U64,
            4 => ValueType::I64,
            5 => ValueType::F64,
            6 => ValueType::Decimal,
            7 => ValueType::I128,
            8 => ValueType::U128,
            9 => ValueType::B32,
            10 => ValueType::B64,
            11 => ValueType::Bytes,
            12 => ValueType::Array,
            13 => ValueType::Map,
            _ => {
                return Err(serde::de::Error::invalid_value(
                    serde::de::Unexpected::Unsigned(v as u64),
                    &"value in [0, 13]",
                ))
            }
        })
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(match v {
            keys::NULL => ValueType::Null,
            keys::STRING => ValueType::String,
            keys::BOOL => ValueType::Bool,
            keys::U64 => ValueType::U64,
            keys::I64 => ValueType::I64,
            keys::F64 => ValueType::F64,
            keys::DECIMAL => ValueType::Decimal,
            keys::I128 => ValueType::I128,
            keys::U128 => ValueType::U128,
            keys::B32 => ValueType::B32,
            keys::B64 => ValueType::B64,
            keys::BYTES => ValueType::Bytes,
            keys::ARRAY => ValueType::Array,
            keys::MAP => ValueType::Map,
            _ => {
                return Err(serde::de::Error::invalid_value(
                    serde::de::Unexpected::Str(v),
                    &"one of valid keys",
                ))
            }
        })
    }
}

impl<'de> serde::Deserialize<'de> for ValueType {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if d.is_human_readable() {
            d.deserialize_str(ValueTypeVisitor)
        } else {
            d.deserialize_u32(ValueTypeVisitor)
        }
    }
}

impl Value {
    pub fn kind(&self) -> ValueType {
        match self {
            Value::Null => ValueType::Null,
            Value::String(_) => ValueType::String,
            Value::Bool(_) => ValueType::Bool,
            Value::U64(_) => ValueType::U64,
            Value::I64(_) => ValueType::I64,
            Value::F64(_) => ValueType::F64,
            Value::Decimal(_) => ValueType::Decimal,
            Value::I128(_) => ValueType::I128,
            Value::U128(_) => ValueType::U128,
            Value::B32(_) => ValueType::B32,
            Value::B64(_) => ValueType::B64,
            Value::Bytes(_) => ValueType::Bytes,
            Value::Array(_) => ValueType::Array,
            Value::Map(_) => ValueType::Map,
        }
    }
}
