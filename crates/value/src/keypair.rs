use crate::tokens;
use serde::ser::SerializeStruct;
use solana_sdk::signer::keypair::Keypair;

pub type Target = Keypair;

pub mod opt {
    pub fn serialize<S>(sig: &Option<super::Target>, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match sig {
            Some(sig) => super::serialize(sig, s),
            None => s.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(d: D) -> Result<Option<super::Target>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        d.deserialize_option(crate::OptionVisitor(super::Visitor))
    }
}

pub fn serialize<S>(k: &Target, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let mut s = s.serialize_struct(tokens::KEYPAIR, 0)?;
    s.serialize_field("", &crate::Bytes(&k.to_bytes()))?;
    s.end()
}

struct Visitor;

impl<'de> serde::de::Visitor<'de> for Visitor {
    type Value = Keypair;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("keypair, or bs58 string")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Keypair::from_bytes(v).map_err(|_| serde::de::Error::invalid_length(v.len(), &"64"))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let mut buf = [0u8; 64];
        let size = bs58::decode(v).into(&mut buf).map_err(|_| {
            serde::de::Error::invalid_value(
                serde::de::Unexpected::Str(v),
                &"keypair encoded in bs58",
            )
        })?;
        self.visit_bytes(&buf[..size])
    }

    fn visit_newtype_struct<D>(self, d: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        d.deserialize_any(self)
    }
}

pub fn deserialize<'de, D>(d: D) -> Result<Target, D::Error>
where
    D: serde::Deserializer<'de>,
{
    d.deserialize_newtype_struct(tokens::KEYPAIR, Visitor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Value;
    use solana_sdk::signer::keypair::Keypair;

    fn de<'de, D: serde::Deserializer<'de>>(d: D) -> Keypair {
        deserialize(d).unwrap()
    }

    fn j(s: &str) -> serde_json::Deserializer<serde_json::de::StrRead> {
        serde_json::Deserializer::from_str(s)
    }

    #[test]
    fn test_deserialize_value() {
        let k = Keypair::new();
        assert_eq!(de(Value::Keypair(k.to_bytes())), k);
        assert_eq!(de(Value::String(k.to_base58_string())), k);
    }

    #[test]
    fn test_deserialize_json() {
        let k = Keypair::new();
        assert_eq!(de(&mut j(&format!("\"{}\"", k.to_base58_string()))), k);
    }
}
