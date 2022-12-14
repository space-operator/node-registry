use crate::prelude::*;
use metaboss_utils::commands::burn::{burn, BurnArgs};

#[derive(Clone, Debug)]
pub struct Burn;

const BURN: &str = "burn";

// Inputs
const KEYPAIR: &str = "keypair";
const MINT_PUBKEY: &str = "mint_account_pubkey";

// Output
const SIGNATURE: &str = "signature";

#[derive(Serialize, Deserialize, Debug)]
pub struct Input {
    #[serde(with = "value::keypair")]
    pub keypair: Keypair,
    #[serde(with = "value::pubkey")]
    pub mint_account_pubkey: Pubkey,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    #[serde(with = "value::signature")]
    pub signature: Signature,
}

#[async_trait]
impl CommandTrait for Burn {
    fn name(&self) -> Name {
        BURN.into()
    }

    fn inputs(&self) -> Vec<CmdInput> {
        [
            CmdInput {
                name: KEYPAIR.into(),
                type_bounds: [ValueType::Keypair, ValueType::String].to_vec(),
                required: true,
                passthrough: false,
            },
            CmdInput {
                name: MINT_PUBKEY.into(),
                type_bounds: [ValueType::Pubkey, ValueType::Keypair, ValueType::String].to_vec(),
                required: true,
                passthrough: false,
            },
        ]
        .to_vec()
    }

    fn outputs(&self) -> Vec<CmdOutput> {
        [CmdOutput {
            name: SIGNATURE.into(),
            r#type: ValueType::Signature,
        }]
        .to_vec()
    }

    async fn run(&self, ctx: Context, inputs: ValueSet) -> Result<ValueSet, CommandError> {
        let input: Input = value::from_map(inputs)?;

        let args = BurnArgs {
            client: &ctx.solana_client,
            keypair: Arc::new(input.keypair),
            mint_pubkey: input.mint_account_pubkey,
        };

        let sig = burn(&args).await.map_err(crate::Error::custom)?;

        Ok(value::to_map(&Output { signature: sig })?)
    }
}

inventory::submit!(CommandDescription::new(BURN, |_| Box::new(Burn)));
