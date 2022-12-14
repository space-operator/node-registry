use crate::prelude::*;
use metaboss_utils::commands::update::{set_immutable, SetImmutableArgs};

#[derive(Clone, Debug)]
pub struct SetImmutable;

const SET_IMMUTABLE: &str = "set_immutable";

// Inputs
const KEYPAIR: &str = "keypair";
const MINT_ACCOUNT: &str = "mint_account";

// Outputs
const SIGNATURE: &str = "signature";

#[derive(Serialize, Deserialize, Debug)]
pub struct Input {
    #[serde(with = "value::keypair")]
    pub keypair: Keypair,
    #[serde(with = "value::pubkey")]
    pub mint_account: Pubkey,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    #[serde(with = "value::signature")]
    pub signature: Signature,
}

#[async_trait]
impl CommandTrait for SetImmutable {
    fn name(&self) -> Name {
        SET_IMMUTABLE.into()
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
                name: MINT_ACCOUNT.into(),
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
            r#type: ValueType::String,
        }]
        .to_vec()
    }

    async fn run(&self, ctx: Context, inputs: ValueSet) -> Result<ValueSet, CommandError> {
        let input: Input = value::from_map(inputs)?;

        let args = SetImmutableArgs {
            client: &ctx.solana_client,
            keypair: Arc::new(input.keypair),
            mint_account: input.mint_account,
        };

        let sig = set_immutable(args).await.map_err(crate::Error::custom)?;

        Ok(value::to_map(&Output { signature: sig })?)
    }
}

inventory::submit!(CommandDescription::new(SET_IMMUTABLE, |_| Box::new(
    SetImmutable
)));
