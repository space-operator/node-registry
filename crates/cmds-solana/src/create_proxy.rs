use space_wrapper::instruction::CreateProxyAuthority;

use crate::{
    prelude::*,
    utils::{sol_to_lamports, tx_to_string},
};

#[derive(Debug, Clone)]
pub struct CreateProxy;

// Command name
const CREATE_PROXY_AUTHORITY: &str = "create_proxy_authority";

// INPUTS
const FEE_PAYER: &str = "fee_payer";
const BUMP: &str = "bump";

#[derive(Serialize, Deserialize, Debug)]
pub struct Input {
    #[serde(with = "value::keypair")]
    pub fee_payer: Keypair,
    #[serde(with = "value::decimal")]
    pub bump: Decimal,
}

// OUTPUTS
const PROXY_AUTHORITY: &str = "proxy_authority";
const SIGNATURE: &str = "signature";

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    #[serde(with = "value::signature")]
    pub signature: Signature,
    #[serde(with = "value::pubkey")]
    pub proxy_authority: Pubkey,
}

#[async_trait]
impl CommandTrait for CreateProxy {
    fn name(&self) -> Name {
        CREATE_PROXY_AUTHORITY.into()
    }

    fn inputs(&self) -> Vec<CmdInput> {
        [
            CmdInput {
                name: FEE_PAYER.into(),
                type_bounds: [ValueType::Keypair, ValueType::String].to_vec(),
                required: true,
                passthrough: false,
            },
            CmdInput {
                name: BUMP.into(),
                type_bounds: [ValueType::Decimal].to_vec(),
                required: true,
                passthrough: false,
            },
        ]
        .to_vec()
    }

    fn outputs(&self) -> Vec<CmdOutput> {
        [
            CmdOutput {
                name: PROXY_AUTHORITY.into(),
                r#type: ValueType::Pubkey,
            },
            CmdOutput {
                name: SIGNATURE.into(),
                r#type: ValueType::String,
            },
        ]
        .to_vec()
    }

    async fn run(&self, ctx: Context, inputs: ValueSet) -> Result<ValueSet, CommandError> {
        let Input { fee_payer, bump } = value::from_map(inputs)?;

        /// 1. Get Instruction
        // let instruction = CreateProxyAuthority;

        /// 2. Create Transaction with execute()
        // let (mut tx, recent_blockhash) =
        //     execute(&ctx.solana_client, &fee_payer.pubkey(), &[instruction], 0).await?;

        /// 3. try_sign_wallet()
        // try_sign_wallet(&ctx, &mut tx, &[&sender], recent_blockhash).await?;

        /// 4. Output Signature
        let signature = Signature::default();

        Ok(value::to_map(&Output {
            proxy_authority,
            signature,
        })?)
    }
}

inventory::submit!(CommandDescription::new(CREATE_PROXY_AUTHORITY, |_| {
    Box::new(CreateProxy)
}));

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::request_airdrop as airdrop;
//     use rust_decimal_macros::dec;

//     #[tokio::test]
//     async fn test_valid() {
//         let ctx = Context::default();

//         let sender = Keypair::from_base58_string("4rQanLxTFvdgtLsGirizXejgYXACawB5ShoZgvz4wwXi4jnii7XHSyUFJbvAk4ojRiEAHvzK6Qnjq7UyJFNbydeQ");
//         let recipient: Pubkey = "GQZRKDqVzM4DXGGMEUNdnBD3CC4TTywh3PwgjYPBm8W9"
//             .parse()
//             .unwrap();

//         // airdrop if necessary
//         let airdrop_output = airdrop::RequestAirdrop
//             .run(
//                 ctx.clone(),
//                 value::to_map(&airdrop::Input {
//                     pubkey: sender.pubkey(),
//                     amount: 1_000_000_000,
//                 })
//                 .unwrap(),
//             )
//             .await;
//         let _ = dbg!(airdrop_output);

//         // Transfer
//         let output = TransferSol
//             .run(
//                 ctx,
//                 value::to_map(&Input {
//                     sender,
//                     recipient,
//                     amount: dec!(0.1),
//                     submit: true,
//                 })
//                 .unwrap(),
//             )
//             .await
//             .unwrap();
//         dbg!(output);
//     }
// }
