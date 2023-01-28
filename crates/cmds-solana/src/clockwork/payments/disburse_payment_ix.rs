use crate::{prelude::*, utils::anchor_sighash};
use anchor_spl::{associated_token, token};
use clockwork_utils::PAYER_PUBKEY;
use payments::state::Payment as ClockworkPayment;
use serde_json::{to_value, Value as JsonValue};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    system_program, sysvar,
};

#[derive(Debug, Clone)]
pub struct DisbursePaymentIx;

#[derive(Serialize, Deserialize, Debug)]
pub struct Input {
    #[serde(with = "value::pubkey")]
    payer: Pubkey,
    #[serde(with = "value::pubkey")]
    authority_token_account: Pubkey,
    #[serde(with = "value::pubkey")]
    mint: Pubkey,
    #[serde(with = "value::pubkey")]
    payment: Pubkey,
    #[serde(with = "value::pubkey")]
    thread: Pubkey,
    #[serde(with = "value::pubkey")]
    recipient: Pubkey,
    #[serde(with = "value::pubkey")]
    recipient_ata: Pubkey,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    instruction: JsonValue,
}

// Name
const DISBURSE_PAYMENT_IX: &str = "disburse_payment_ix";

// Inputs
const PAYER: &str = "pubkey";
const AUTHORITY_TOKEN_ACCOUNT: &str = "pubkey";
const MINT: &str = "pubkey";
const PAYMENT: &str = "pubkey";
const THREAD: &str = "pubkey";
const RECIPIENT: &str = "pubkey";
const RECIPIENT_ATA: &str = "pubkey";

// Outputs
const INSTRUCTION: &str = "instruction";

#[async_trait]
impl CommandTrait for DisbursePaymentIx {
    fn name(&self) -> Name {
        DISBURSE_PAYMENT_IX.into()
    }

    fn inputs(&self) -> Vec<CmdInput> {
        [
            CmdInput {
                name: PAYER.into(),
                type_bounds: [ValueType::Pubkey, ValueType::Keypair, ValueType::String].to_vec(),
                required: true,
                passthrough: true,
            },
            CmdInput {
                name: AUTHORITY_TOKEN_ACCOUNT.into(),
                type_bounds: [ValueType::Pubkey, ValueType::Keypair, ValueType::String].to_vec(),
                required: true,
                passthrough: true,
            },
            CmdInput {
                name: MINT.into(),
                type_bounds: [ValueType::Pubkey, ValueType::Keypair, ValueType::String].to_vec(),
                required: true,
                passthrough: true,
            },
            CmdInput {
                name: PAYMENT.into(),
                type_bounds: [ValueType::Pubkey, ValueType::Keypair, ValueType::String].to_vec(),
                required: true,
                passthrough: true,
            },
            CmdInput {
                name: THREAD.into(),
                type_bounds: [ValueType::Pubkey, ValueType::Keypair, ValueType::String].to_vec(),
                required: true,
                passthrough: true,
            },
            CmdInput {
                name: RECIPIENT.into(),
                type_bounds: [ValueType::Pubkey, ValueType::Keypair, ValueType::String].to_vec(),
                required: true,
                passthrough: true,
            },
            CmdInput {
                name: RECIPIENT_ATA.into(),
                type_bounds: [ValueType::Pubkey, ValueType::Keypair, ValueType::String].to_vec(),
                required: true,
                passthrough: true,
            },
        ]
        .to_vec()
    }

    fn outputs(&self) -> Vec<CmdOutput> {
        [CmdOutput {
            name: INSTRUCTION.into(),
            r#type: ValueType::Json,
        }]
        .to_vec()
    }

    async fn run(&self, ctx: Context, inputs: ValueSet) -> Result<ValueSet, CommandError> {
        let Input {
            payer,
            authority_token_account,
            mint,
            payment,
            thread,
            recipient,
            recipient_ata,
        } = value::from_map::<Input>(inputs.clone())?;
        dbg!(&inputs);

        let payment = ClockworkPayment::pubkey(payer, mint, recipient);

        let program_id = payments::ID;
        let accounts = vec![
            AccountMeta::new_readonly(associated_token::ID, false),
            AccountMeta::new_readonly(payer, false),
            AccountMeta::new(authority_token_account, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new(PAYER_PUBKEY, true),
            AccountMeta::new(payment, false),
            AccountMeta::new(thread, true),
            AccountMeta::new_readonly(recipient, false),
            AccountMeta::new(recipient_ata, false),
            AccountMeta::new_readonly(sysvar::rent::ID, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(token::ID, false),
        ];
        let data = anchor_sighash("disburse_payment").to_vec();

        let instruction = Instruction::new_with_bytes(program_id, &data, accounts);
        dbg!(&instruction);

        let instruction = to_value(&instruction).unwrap();
        dbg!(&instruction);

        Ok(value::to_map(&Output { instruction })?)
    }
}

inventory::submit!(CommandDescription::new(DISBURSE_PAYMENT_IX, |_| Box::new(
    DisbursePaymentIx {}
)));

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_valid() {}
}
