use crate::prelude::*;

#[derive(Debug)]
pub struct RequestAirdrop;

fn default_amount() -> u64 {
    1000000000
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Input {
    #[serde(with = "value::pubkey")]
    pub pubkey: Pubkey,
    #[serde(default = "default_amount")]
    pub amount: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    #[serde(with = "value::signature")]
    pub signature: Signature,
}

const SOLANA_REQUEST_AIRDROP: &str = "request_airdrop";

// Inputs
const PUBKEY: &str = "pubkey";
const AMOUNT: &str = "amount";

// Outputs
const SIGNATURE: &str = "signature";

#[async_trait]
impl CommandTrait for RequestAirdrop {
    fn name(&self) -> Name {
        SOLANA_REQUEST_AIRDROP.into()
    }

    fn inputs(&self) -> Vec<CmdInput> {
        [
            CmdInput {
                name: PUBKEY.into(),
                type_bounds: [ValueType::Pubkey].to_vec(),
                required: true,
                passthrough: true,
            },
            CmdInput {
                name: AMOUNT.into(),
                type_bounds: [ValueType::U64].to_vec(),
                required: false,
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

        let signature = ctx
            .solana_client
            .request_airdrop(&input.pubkey, input.amount)
            .await?;

        Ok(value::to_map(&Output { signature })?)
    }
}

inventory::submit!(CommandDescription::new(SOLANA_REQUEST_AIRDROP, |_| {
    Ok(Box::new(RequestAirdrop))
}));

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_valid() {
        let pubkey: Pubkey = "DKsvmM9hfNm4R94yB3VdYMZJk2ETv5hpcjuRmiwgiztY"
            .parse()
            .unwrap();
        let amount: u64 = 1_500_000_000;

        let input = value::to_map(&Input { amount, pubkey }).unwrap();
        let result = RequestAirdrop.run(Context::default(), input).await;
        dbg!(&result);
        // check for balance https://explorer.solana.com/address/DKsvmM9hfNm4R94yB3VdYMZJk2ETv5hpcjuRmiwgiztY?cluster=devnet

        // TODO: Get the error handling here figured out.
        // The issue is that we either get rate-limited or succeed.
    }
}
