use crate::prelude::*;
use bip39::{Language, Mnemonic, MnemonicType, Seed};
use solana_sdk::signature::{keypair_from_seed, Keypair};
use solana_sdk::signer::Signer;

#[derive(Debug, Clone)]
pub struct GenerateKeypair;

const GENERATE_KEYPAIR: &str = "generate_keypair";

fn random_seed() -> String {
    Mnemonic::new(MnemonicType::Words12, Language::English).into_phrase()
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Input {
    PrivateKey {
        #[serde(with = "value::keypair")]
        private_key: Keypair,
    },
    Seed {
        #[serde(default = "random_seed")]
        seed: String,
        #[serde(default)]
        passphrase: String,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    #[serde(with = "value::pubkey")]
    pub pubkey: Pubkey,
    #[serde(with = "value::keypair")]
    pub keypair: Keypair,
}

const SEED: &str = "seed";
const PRIVATE_KEY: &str = "private_key";
const PASSPHRASE: &str = "passphrase";

const PUBKEY: &str = "pubkey";
const KEYPAIR: &str = "keypair";

#[async_trait]
impl CommandTrait for GenerateKeypair {
    fn name(&self) -> Name {
        GENERATE_KEYPAIR.into()
    }

    fn inputs(&self) -> Vec<CmdInput> {
        [
            CmdInput {
                name: SEED.into(),
                type_bounds: [ValueType::String].to_vec(),
                required: false,
                passthrough: false,
            },
            CmdInput {
                name: PRIVATE_KEY.into(),
                type_bounds: [ValueType::String].to_vec(),
                required: false,
                passthrough: false,
            },
            CmdInput {
                name: PASSPHRASE.into(),
                type_bounds: [ValueType::String].to_vec(),
                required: false,
                passthrough: false,
            },
        ]
        .to_vec()
    }

    fn outputs(&self) -> Vec<CmdOutput> {
        [
            CmdOutput {
                name: PUBKEY.into(),
                r#type: ValueType::Pubkey,
            },
            CmdOutput {
                name: KEYPAIR.into(),
                r#type: ValueType::Keypair,
            },
        ]
        .to_vec()
    }

    async fn run(&self, _: Context, inputs: ValueSet) -> Result<ValueSet, CommandError> {
        let keypair = match value::from_map(inputs)? {
            Input::PrivateKey { private_key } => private_key,
            Input::Seed { seed, passphrase } => generate_keypair(&passphrase, &seed)?,
        };

        Ok(value::to_map(&Output {
            pubkey: keypair.pubkey(),
            keypair,
        })?)
    }
}

fn generate_keypair(passphrase: &str, seed: &str) -> crate::Result<Keypair> {
    let sanitized = seed.split_whitespace().collect::<Vec<&str>>().join(" ");
    let parse_language_fn = || {
        for language in &[
            Language::English,
            Language::ChineseSimplified,
            Language::ChineseTraditional,
            Language::Japanese,
            Language::Spanish,
            Language::Korean,
            Language::French,
            Language::Italian,
        ] {
            if let Ok(mnemonic) = Mnemonic::from_phrase(&sanitized, *language) {
                return Ok(mnemonic);
            }
        }
        Err(crate::Error::CantGetMnemonicFromPhrase)
    };
    let mnemonic = parse_language_fn()?;
    let seed = Seed::new(&mnemonic, passphrase);
    keypair_from_seed(seed.as_bytes()).map_err(|e| crate::Error::KeypairFromSeed(e.to_string()))
}

inventory::submit!(CommandDescription::new(GENERATE_KEYPAIR, |_| Box::new(
    GenerateKeypair
)));

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_no_input() {
        let ctx = Context::default();
        GenerateKeypair.run(ctx, ValueSet::new()).await.unwrap();
    }

    #[tokio::test]
    async fn test_no_password() {
        let seed_phrase =
            "letter advice cage absurd amount doctor acoustic avoid letter advice cage above";
        let ctx = Context::default();
        GenerateKeypair
            .run(
                ctx,
                value::map! {
                    "seed" => Value::String(seed_phrase.to_owned()),
                },
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_private_key() {
        let private_key =
            "56Ngo8EY5ZWmYKDZAmKYcUf2y2LZVRSMMnptGp9JtQuSZHyU3Pwhhkmj5YVf89VTQZqrzkabhybWdWwJWCa74aYu";
        let input = value::map! {
            "private_key" => Value::new_keypair_bs58(private_key).unwrap(),
        };
        let cmd = GenerateKeypair;
        let output = cmd.run(Context::default(), input).await.unwrap();
        let output = value::from_map::<Output>(output).unwrap();
        assert_eq!(output.keypair.to_base58_string(), "56Ngo8EY5ZWmYKDZAmKYcUf2y2LZVRSMMnptGp9JtQuSZHyU3Pwhhkmj5YVf89VTQZqrzkabhybWdWwJWCa74aYu");
        assert_eq!(
            output.pubkey.to_string(),
            "GQZRKDqVzM4DXGGMEUNdnBD3CC4TTywh3PwgjYPBm8W9"
        );
    }

    #[tokio::test]
    async fn test_keypair_b() {
        let private_key =
            "4rQanLxTFvdgtLsGirizXejgYXACawB5ShoZgvz4wwXi4jnii7XHSyUFJbvAk4ojRiEAHvzK6Qnjq7UyJFNbydeQ";
        let input = value::map! {
            "private_key" => Value::String(private_key.to_owned()),
        };
        let output = GenerateKeypair
            .run(Context::default(), input)
            .await
            .unwrap();
        let output = value::from_map::<Output>(output).unwrap();
        assert_eq!(output.keypair.to_base58_string(), private_key);
    }

    #[tokio::test]
    async fn test_seed_and_pass() {
        let seed_phrase =
            "letter advice cage absurd amount doctor acoustic avoid letter advice cage above";
        let passphrase = "Hunter1!";

        let keypair = generate_keypair(passphrase, seed_phrase).unwrap();

        let input = value::map! {
            "seed" => Value::String(seed_phrase.to_owned()),
            "passphrase" => Value::String(passphrase.to_owned()),
        };
        let output = GenerateKeypair
            .run(Context::default(), input)
            .await
            .unwrap();
        let output = value::from_map::<Output>(output).unwrap();
        assert_eq!(
            output.pubkey.to_string(),
            "ESxeViFP4r7THzVx9hJDkhj4HrNGSjJSFRPbGaAb97hN"
        );
        assert_eq!(
            output.keypair.to_base58_string(),
            "3LUpzbebV5SCftt8CPmicbKxNtQhtJegEz4n8s6LBf3b1s4yfjLapgJhbMERhP73xLmWEP2XJ2Rz7Y3TFiYgTpXv"
        );
        assert_eq!(output.pubkey, keypair.pubkey(),);
        assert_eq!(output.keypair, keypair,);
    }

    #[tokio::test]
    async fn test_invalid() {
        let seed_phrase =
            "letter advice cage absurd amount doctor acoustic avoid letter advice cage above";
        let passphrase = "Hunter1!";
        let private_key =
            "4rQanLxTFvdgtLsGirizXejgY5ShoZgvz4wwXi4jnii7XHSyUFJbvAk4ojRiEAHvzK6Qnjq7UyJFNbydeQ";
        let input = value::map! {
            "seed" => Value::String(seed_phrase.to_owned()),
            "passphrase" => Value::String(passphrase.to_owned()),
            "private_key" => Value::String(private_key.to_string()),
        };
        let output = GenerateKeypair
            .run(Context::default(), input)
            .await
            .unwrap();
        let output = value::from_map::<Output>(output).unwrap();
        assert_eq!(
            output.keypair,
            generate_keypair(passphrase, seed_phrase).unwrap(),
        );
    }
}
