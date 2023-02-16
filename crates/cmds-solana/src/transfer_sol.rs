use crate::{
    prelude::*,
    utils::{sol_to_lamports, tx_to_string},
};

#[derive(Debug, Clone)]
pub struct TransferSol;

#[derive(Serialize, Deserialize, Debug)]
pub struct Input {
    #[serde(with = "value::keypair")]
    pub sender: Keypair,
    #[serde(with = "value::pubkey")]
    pub recipient: Pubkey,
    #[serde(with = "value::decimal")]
    pub amount: Decimal,
    #[serde(default = "value::default::bool_true")]
    pub submit: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    #[serde(default, with = "value::signature::opt")]
    pub signature: Option<Signature>,
    pub tx: String,
}

const SOLANA_TRANSFER_SOL: &str = "transfer_sol";

// Inputs
const SENDER: &str = "sender";
const RECIPIENT: &str = "recipient";
const AMOUNT: &str = "amount";
const SUBMIT: &str = "submit";

// Outputs
const TX: &str = "tx";
const SIGNATURE: &str = "signature";

#[async_trait]
impl CommandTrait for TransferSol {
    fn name(&self) -> Name {
        SOLANA_TRANSFER_SOL.into()
    }

    fn inputs(&self) -> Vec<CmdInput> {
        [
            CmdInput {
                name: SENDER.into(),
                type_bounds: [ValueType::Keypair, ValueType::String].to_vec(),
                required: true,
                passthrough: false,
            },
            CmdInput {
                name: RECIPIENT.into(),
                type_bounds: [ValueType::Pubkey, ValueType::Keypair, ValueType::String].to_vec(),
                required: true,
                passthrough: false,
            },
            CmdInput {
                name: AMOUNT.into(),
                type_bounds: [ValueType::F64].to_vec(),
                required: true,
                passthrough: false,
            },
            CmdInput {
                name: SUBMIT.into(),
                type_bounds: [ValueType::Bool].to_vec(),
                required: false,
                passthrough: false,
            },
        ]
        .to_vec()
    }

    fn outputs(&self) -> Vec<CmdOutput> {
        [
            CmdOutput {
                name: SIGNATURE.into(),
                r#type: ValueType::String,
            },
            CmdOutput {
                name: TX.into(),
                r#type: ValueType::String,
            },
        ]
        .to_vec()
    }

    async fn run(&self, ctx: Context, inputs: ValueSet) -> Result<ValueSet, CommandError> {
        let Input {
            sender,
            recipient,
            amount,
            submit,
        } = value::from_map(inputs)?;
        let amount = sol_to_lamports(amount)?;

        let instruction =
            solana_sdk::system_instruction::transfer(&sender.pubkey(), &recipient, amount);
        let (mut tx, recent_blockhash) =
            execute(&ctx.solana_client, &sender.pubkey(), &[instruction], 0).await?;

        try_sign_wallet(&ctx, &mut tx, &[&sender], recent_blockhash).await?;

        let tx_str = tx_to_string(&tx)?;

        let signature = if submit {
            Some(submit_transaction(&ctx.solana_client, tx).await?)
        } else {
            None
        };

        Ok(value::to_map(&Output {
            signature,
            tx: tx_str,
        })?)
    }
}

inventory::submit!(CommandDescription::new(SOLANA_TRANSFER_SOL, |_| Box::new(
    TransferSol
)));

#[cfg(test)]
mod tests {
    use super::*;
    use crate::request_airdrop as airdrop;
    use rust_decimal_macros::dec;

    #[tokio::test]
    async fn test_valid() {
        let ctx = Context::default();

        let sender = Keypair::from_base58_string("4rQanLxTFvdgtLsGirizXejgYXACawB5ShoZgvz4wwXi4jnii7XHSyUFJbvAk4ojRiEAHvzK6Qnjq7UyJFNbydeQ");
        let recipient: Pubkey = "GQZRKDqVzM4DXGGMEUNdnBD3CC4TTywh3PwgjYPBm8W9"
            .parse()
            .unwrap();

        // airdrop if necessary
        let airdrop_output = airdrop::RequestAirdrop
            .run(
                ctx.clone(),
                value::to_map(&airdrop::Input {
                    pubkey: sender.pubkey(),
                    amount: 1_000_000_000,
                })
                .unwrap(),
            )
            .await;
        let _ = dbg!(airdrop_output);

        // Transfer
        let output = TransferSol
            .run(
                ctx,
                value::to_map(&Input {
                    sender,
                    recipient,
                    amount: dec!(0.1),
                    submit: true,
                })
                .unwrap(),
            )
            .await
            .unwrap();
        dbg!(output);
    }
}
