use crate::prelude::*;
use metaboss_utils::commands::update::{set_primary_sale_happened, SetPrimarySaleHappenedArgs};

#[derive(Clone, Debug)]
pub struct PrimarySaleHappened;

const PRIMARY_SALE_HAPPENED: &str = "primary_sale_happened";

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
impl CommandTrait for PrimarySaleHappened {
    fn name(&self) -> Name {
        PRIMARY_SALE_HAPPENED.into()
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

        let args = SetPrimarySaleHappenedArgs {
            client: &ctx.solana_client,
            keypair: Arc::new(input.keypair),
            mint_account: input.mint_account,
        };

        let sig = set_primary_sale_happened(&args)
            .await
            .map_err(crate::Error::custom)?;

        Ok(value::to_map(&Output { signature: sig })?)
    }
}

inventory::submit!(CommandDescription::new(
    PRIMARY_SALE_HAPPENED,
    |_| Box::new(PrimarySaleHappened)
));
