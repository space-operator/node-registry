use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct GetBalance;

#[derive(Serialize, Deserialize, Debug)]
pub struct Input {
    #[serde(with = "value::pubkey")]
    pubkey: Pubkey,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    balance: u64,
}

// Name
const GET_BALANCE: &str = "get_balance";

// Inputs
const PUBKEY: &str = "pubkey";

// Outputs
const BALANCE: &str = "balance";

#[async_trait]
impl CommandTrait for GetBalance {
    fn name(&self) -> Name {
        GET_BALANCE.into()
    }

    fn inputs(&self) -> Vec<CmdInput> {
        [CmdInput {
            name: PUBKEY.into(),
            type_bounds: [ValueType::Pubkey, ValueType::Keypair, ValueType::String].to_vec(),
            required: true,
            passthrough: false,
        }]
        .to_vec()
    }

    fn outputs(&self) -> Vec<CmdOutput> {
        [CmdOutput {
            name: BALANCE.into(),
            r#type: ValueType::U64,
        }]
        .to_vec()
    }

    async fn run(&self, ctx: Context, inputs: ValueSet) -> Result<ValueSet, CommandError> {
        let pubkey = value::from_map::<Input>(inputs)?.pubkey;

        let balance = ctx.solana_client.get_balance(&pubkey).await?;

        Ok(value::to_map(&Output { balance })?)
    }
}

inventory::submit!(CommandDescription::new(GET_BALANCE, |_| Box::new(
    GetBalance {}
)));

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_valid() {
        let input = value::to_map(&Input {
            pubkey: Pubkey::new_from_array([1; 32]),
        })
        .unwrap();
        let output = GetBalance.run(Context::default(), input).await.unwrap();
        let balance = value::from_map::<Output>(output).unwrap().balance;
        dbg!(balance);
    }
}
