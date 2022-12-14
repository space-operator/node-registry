use crate::prelude::*;
use metaboss_utils::commands::update::update_name;

#[derive(Debug)]
pub struct UpdateName;

const UPDATE_NAME: &str = "update_name";

// Inputs
const KEYPAIR: &str = "keypair";
const MINT_ACCOUNT: &str = "mint_account";
const NAME: &str = "name";

// Outputs
const SIGNATURE: &str = "signature";

#[derive(Serialize, Deserialize, Debug)]
pub struct Input {
    #[serde(with = "value::keypair")]
    pub keypair: Keypair,
    #[serde(with = "value::pubkey")]
    pub mint_account: Pubkey,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    #[serde(with = "value::signature")]
    pub signature: Signature,
}

#[async_trait]
impl CommandTrait for UpdateName {
    fn name(&self) -> Name {
        UPDATE_NAME.into()
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
            CmdInput {
                name: NAME.into(),
                type_bounds: [ValueType::String].to_vec(),
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

        let sig = update_name(
            &ctx.solana_client,
            input.keypair,
            &input.mint_account,
            &input.name,
        )
        .await
        .map_err(crate::Error::custom)?;

        Ok(value::to_map(&Output { signature: sig })?)
    }
}

inventory::submit!(CommandDescription::new(UPDATE_NAME, |_| Box::new(
    UpdateName
)));
