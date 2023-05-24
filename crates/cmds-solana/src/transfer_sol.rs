use bytes::Bytes;

use crate::{prelude::*, utils::sol_to_lamports};

const TRANSFER_SOL: &str = "transfer_sol";

const DEFINITION: &str = include_str!("../../../node-definitions/solana/transfer_sol.json");

fn build() -> Result<Box<dyn CommandTrait>, CommandError> {
    use once_cell::sync::Lazy;
    static CACHE: Lazy<Result<CmdBuilder, BuilderError>> = Lazy::new(|| {
        CmdBuilder::new(DEFINITION)?
            .check_name(TRANSFER_SOL)?
            .simple_instruction_info("signature")
    });
    Ok(CACHE.clone()?.build(run))
}

inventory::submit!(CommandDescription::new(TRANSFER_SOL, |_| { build() }));

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
}

async fn run(mut ctx: Context, input: Input) -> Result<Output, CommandError> {
    let amount = sol_to_lamports(input.amount)?;

    let instruction =
        solana_sdk::system_instruction::transfer(&input.sender.pubkey(), &input.recipient, amount);

    let minimum_balance_for_rent_exemption = ctx
        .solana_client
        .get_minimum_balance_for_rent_exemption(24)
        .await?;

    // Bundle it all up
    let ins = Instructions {
        fee_payer: input.sender.pubkey(),
        signers: [input.sender.clone_keypair()].into(),
        instructions: [instruction].into(),
        minimum_balance_for_rent_exemption,
    };

    let instructions = ins
        .instructions
        .clone()
        .into_iter()
        .map(|i| {
            Value::Map(value::map! {
                "program_id" => i.program_id,
                "accounts" => i.accounts.into_iter().map(|a| Value::Map(value::map! {
                    "pubkey" => a.pubkey,
                    "is_signer" => a.is_signer,
                    "is_writable" => a.is_writable,
                })).collect::<Vec<_>>(),
                "data" => Bytes::from(i.data),
            })
        })
        .collect::<Vec<_>>();

    let ins = input.submit.then_some(ins).unwrap_or_default();

    let signature = ctx.execute(ins,    value::map! {
        "instructions" => instructions
    },).await?.signature;

    Ok(Output { signature })
}

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
        let output = run(
            ctx,
            Input {
                sender,
                recipient,
                amount: dec!(1030),
                submit: true,
            },
        )
        .await
        .unwrap();
        dbg!(output);
    }
}
