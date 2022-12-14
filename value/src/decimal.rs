use crate::tokens;
use rust_decimal::Decimal;
use serde::ser::SerializeStruct;

pub fn serialize<S>(d: &Decimal, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let mut s = s.serialize_struct(tokens::DECIMAL, 0)?;
    s.serialize_field("", &crate::Bytes(&d.serialize()))?;
    s.end()
}

struct DecimalVisitor;

impl<'de> serde::de::Visitor<'de> for DecimalVisitor {
    type Value = Decimal;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("decimal")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if v.len() != 16 {
            return Err(serde::de::Error::invalid_length(v.len(), &"16"));
        }

        let buf: [u8; 16] = v.try_into().unwrap();
        Ok(Decimal::deserialize(buf))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Decimal::from(v))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Decimal::from(v))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Decimal::try_from(v).map_err(serde::de::Error::custom)
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        v.parse().map_err(serde::de::Error::custom)
    }

    fn visit_newtype_struct<D>(self, d: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        d.deserialize_any(self)
    }
}

pub fn deserialize<'de, D>(d: D) -> Result<Decimal, D::Error>
where
    D: serde::Deserializer<'de>,
{
    d.deserialize_newtype_struct(tokens::DECIMAL, DecimalVisitor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Value;
    use rust_decimal_macros::dec;

    fn de<'de, D: serde::Deserializer<'de>>(d: D) -> Decimal {
        deserialize(d).unwrap()
    }

    fn j(s: &str) -> serde_json::Deserializer<serde_json::de::StrRead> {
        serde_json::Deserializer::from_str(s)
    }

    #[test]
    fn test_deserialize_value() {
        assert_eq!(de(Value::U32(100)), dec!(100));
        assert_eq!(de(Value::I32(-1)), dec!(-1));
        assert_eq!(de(Value::Decimal(Decimal::MAX)), Decimal::MAX);
        assert_eq!(de(Value::F64(1231.2221)), dec!(1231.2221));
        assert_eq!(de(Value::String("1234.0".to_owned())), dec!(1234));
    }

    #[test]
    fn test_deserialize_json() {
        assert_eq!(de(&mut j("100")), dec!(100));
        assert_eq!(de(&mut j("\"100\"")), dec!(100));
        assert_eq!(de(&mut j("100.1234")), dec!(100.1234));
        // assert_eq!(de(&mut j("79228162514264337593543950335")), Decimal::MAX);
    }
}
