use crate::prelude::*;
use metaboss_utils::commands::update::update_symbol;

#[derive(Clone, Debug)]
pub struct UpdateSymbol;

const UPDATE_SYMBOL: &str = "update_symbol";

// Inputs
const KEYPAIR: &str = "keypair";
const MINT_ACCOUNT: &str = "mint_account";
const SYMBOL: &str = "symbol";

// Outputs
const SIGNATURE: &str = "signature";

#[derive(Serialize, Deserialize, Debug)]
pub struct Input {
    #[serde(with = "value::keypair")]
    pub keypair: Keypair,
    #[serde(with = "value::pubkey")]
    pub mint_account: Pubkey,
    pub symbol: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    #[serde(with = "value::signature")]
    pub signature: Signature,
}

#[async_trait]
impl CommandTrait for UpdateSymbol {
    fn name(&self) -> Name {
        UPDATE_SYMBOL.into()
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
                name: SYMBOL.into(),
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

        let sig = update_symbol(
            &ctx.solana_client,
            input.keypair,
            &input.mint_account,
            &input.symbol,
        )
        .await
        .map_err(crate::Error::custom)?;

        Ok(value::to_map(&Output { signature: sig })?)
    }
}

inventory::submit!(CommandDescription::new(UPDATE_SYMBOL, |_| Box::new(
    UpdateSymbol
)));
