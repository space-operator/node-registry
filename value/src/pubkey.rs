use crate::tokens;
use serde::ser::SerializeStruct;
use solana_sdk::pubkey::Pubkey;

pub type Target = Pubkey;

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

pub fn serialize<S>(p: &Target, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let mut s = s.serialize_struct(tokens::PUBKEY, 0)?;
    s.serialize_field("", &crate::Bytes(&p.to_bytes()))?;
    s.end()
}

struct Visitor;

impl<'de> serde::de::Visitor<'de> for Visitor {
    type Value = Target;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("pubkey, keypair, or bs58 string")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match v.len() {
            32 => Ok(Pubkey::new(v)),
            // see ed25519-dalek's Keypair
            64 => Ok(Pubkey::new(&v[32..])),
            l => Err(serde::de::Error::invalid_length(l, &"32 or 64")),
        }
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let mut buf = [0u8; 64];
        let size = bs58::decode(v).into(&mut buf).map_err(|_| {
            serde::de::Error::invalid_value(
                serde::de::Unexpected::Str(v),
                &"pubkey or keypair encoded in bs58",
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
    d.deserialize_newtype_struct(tokens::PUBKEY, Visitor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Value;
    use solana_sdk::pubkey::Pubkey;
    use solana_sdk::signature::Signer;
    use solana_sdk::signer::keypair::Keypair;

    fn de<'de, D: serde::Deserializer<'de>>(d: D) -> Pubkey {
        deserialize(d).unwrap()
    }

    fn j(s: &str) -> serde_json::Deserializer<serde_json::de::StrRead> {
        serde_json::Deserializer::from_str(s)
    }

    #[test]
    fn test_deserialize_value() {
        let id = solana_sdk::feature_set::add_set_compute_unit_price_ix::id();
        assert_eq!(de(Value::String(id.to_string())), id,);
        assert_eq!(de(Value::Pubkey(id)), id);

        let k = Keypair::new();
        let pk = k.pubkey();
        assert_eq!(de(Value::Keypair(k.to_bytes())), pk);
    }

    #[test]
    fn test_deserialize_json() {
        let id: Pubkey = "98std1NSHqXi9WYvFShfVepRdCoq1qvsp8fsR2XZtG8g"
            .parse()
            .unwrap();
        assert_eq!(
            de(&mut j("\"98std1NSHqXi9WYvFShfVepRdCoq1qvsp8fsR2XZtG8g\"")),
            id
        );

        let k = Keypair::new();
        let pk = k.pubkey();
        assert_eq!(de(&mut j(&format!("\"{}\"", k.to_base58_string()))), pk);
    }
}
