use super::Serializer;
use crate::{Error, Value};

pub enum SerializeSeq {
    Bytes(Vec<u8>),
    Array(Vec<Value>),
}

impl Default for SerializeSeq {
    fn default() -> Self {
        Self::new()
    }
}

impl SerializeSeq {
    pub fn new() -> Self {
        SerializeSeq::Bytes(Vec::new())
    }
}

impl TryFrom<Value> for u8 {
    type Error = Value;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::U64(x) => u8::try_from(x).map_err(|_| value),
            Value::I64(x) => u8::try_from(x).map_err(|_| value),
            Value::U128(x) => u8::try_from(x).map_err(|_| value),
            value => Err(value),
        }
    }
}

impl serde::ser::SerializeSeq for SerializeSeq {
    type Ok = Value;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ?Sized + serde::Serialize,
    {
        match self {
            Self::Array(vec) => {
                let value = value.serialize(Serializer)?;
                vec.push(value);
            }
            Self::Bytes(vec) => {
                let value = value.serialize(Serializer)?;
                match u8::try_from(value) {
                    Ok(v) => vec.push(v),
                    Err(v) => {
                        let Self::Bytes(old) = std::mem::replace(self, Self::Array(Vec::new())) else { unreachable!() };
                        let Self::Array(new) = self else { unreachable!() };
                        new.extend(old.into_iter().map(Value::from).chain(std::iter::once(v)));
                    }
                }
            }
        }
        Ok(())
    }

    fn end(self) -> Result<Value, Error> {
        Ok(match self {
            Self::Bytes(vec) => {
                if vec.is_empty() {
                    Value::Array(Vec::new())
                } else {
                    Value::from(vec)
                }
            }
            Self::Array(vec) => Value::Array(vec),
        })
    }
}

impl serde::ser::SerializeTuple for SerializeSeq {
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

impl serde::ser::SerializeTupleStruct for SerializeSeq {
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
