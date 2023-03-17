use crate::prelude::*;
use anchor_lang::{solana_program::sysvar, InstructionData};
use anchor_spl::{associated_token, token};
use clockwork_client::thread::{
    instruction::thread_create,
    state::{Thread, Trigger},
};
use clockwork_thread_program::state::PAYER_PUBKEY;
use clockwork_utils::explorer::Explorer;
use clockwork_utils::thread::SerializableInstruction as ClockWorkInstruction;
use payments::state::Payment as ClockworkPayment;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    system_program,
};
use solana_sdk::pubkey::Pubkey;
use spl_associated_token_account::get_associated_token_address;

#[derive(Debug)]
pub struct Payment;

fn create_payment(
    payer: Pubkey,
    authority_token_account: Pubkey,
    mint: Pubkey,
    payment: Pubkey,
    recipient: Pubkey,
    amount: u64,
) -> Instruction {
    Instruction {
        program_id: payments::ID,
        accounts: vec![
            AccountMeta::new_readonly(associated_token::ID, false),
            AccountMeta::new(payer, true),
            AccountMeta::new(authority_token_account, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new(payment, false),
            AccountMeta::new_readonly(recipient, false),
            AccountMeta::new_readonly(sysvar::rent::ID, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(token::ID, false),
        ],
        data: payments::instruction::CreatePayment { amount }.data(),
    }
}

fn distribute_payment(
    payer: Pubkey,
    authority_token_account: Pubkey,
    mint: Pubkey,
    payment: Pubkey,
    thread: Pubkey,
    recipient: Pubkey,
    recipient_ata_pubkey: Pubkey,
) -> ClockWorkInstruction {
    let distribute_payment_ix = Instruction {
        program_id: payments::ID,
        accounts: vec![
            AccountMeta::new_readonly(associated_token::ID, false),
            AccountMeta::new_readonly(payer, false),
            AccountMeta::new(authority_token_account, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new(PAYER_PUBKEY, true),
            AccountMeta::new(payment, false),
            AccountMeta::new(thread, true),
            AccountMeta::new_readonly(recipient, false),
            AccountMeta::new(recipient_ata_pubkey, false),
            AccountMeta::new_readonly(sysvar::rent::ID, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(token::ID, false),
        ],
        data: anchor_sighash("disburse_payment").to_vec(), // payments::instruction::DisbursePayment.data(),
    };
    distribute_payment_ix.into()
}

pub fn thread_delete(authority: Pubkey, thread: Pubkey) -> Instruction {
    Instruction {
        program_id: clockwork_thread_program::ID,
        accounts: vec![
            AccountMeta::new_readonly(authority, true),
            AccountMeta::new(authority, false),
            AccountMeta::new(thread, false),
        ],
        data: anchor_sighash("thread_delete").to_vec(),
    }
}

impl Payment {
    #[allow(clippy::too_many_arguments)]
    async fn command_create_payment(
        &self,
        rpc_client: &RpcClient,
        payer: Pubkey,
        authority_token_account: Pubkey,
        mint: Pubkey,
        payment: Pubkey,
        thread: Pubkey,
        recipient: Pubkey,
        recipient_ata_pubkey: Pubkey,
        amount: u64,
        trigger: Trigger,
    ) -> crate::Result<(u64, Vec<Instruction>)> {
        // FIXME min rent
        let minimum_balance_for_rent_exemption = rpc_client
            .get_minimum_balance_for_rent_exemption(816 + 608 + 512)
            .await?;

        let distribute_payment_ix = distribute_payment(
            payer,
            authority_token_account,
            mint,
            payment,
            thread,
            recipient,
            recipient_ata_pubkey,
        );

        let instructions = vec![
            // thread_delete(payer, thread),
            create_payment(
                payer,
                authority_token_account,
                mint,
                payment,
                recipient,
                amount,
            ),
            thread_create(
                amount,
                payer,
                "payment".into(),
                vec![distribute_payment_ix],
                payer,
                thread,
                trigger,
            ),
        ];

        Ok((minimum_balance_for_rent_exemption, instructions))
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Input {
    IsImmediate {
        is_immediate: bool,
    },
    Schedule {
        schedule: String,
        is_skippable: bool,
    },
    MonitorAccount {
        #[serde(with = "value::pubkey")]
        monitor_account: Pubkey,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InputStruct {
    #[serde(with = "value::keypair")]
    pub payer: Keypair,
    #[serde(with = "value::pubkey")]
    pub token_account: Pubkey,
    #[serde(with = "value::pubkey")]
    pub token_mint: Pubkey,
    #[serde(with = "value::pubkey")]
    pub recipient: Pubkey,
    pub amount: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    #[serde(with = "value::signature")]
    signature: Signature,
    #[serde(with = "value::pubkey")]
    thread: Pubkey,
}

// Command Name
const CREATE_PAYMENT: &str = "create_payment";

// Inputs
const IS_IMMEDIATE: &str = "is_immediate";
const SCHEDULE: &str = "schedule";
const IS_SKIPPABLE: &str = "is_skippable";
const MONITOR_ACCOUNT: &str = "monitor_account";
const PAYER: &str = "payer";
const TOKEN_ACCOUNT: &str = "token_account";
const TOKEN_MINT: &str = "token_mint";
const RECIPIENT: &str = "recipient";
const AMOUNT: &str = "amount";

// Outputs
const SIGNATURE: &str = "signature";
const THREAD: &str = "thread";

// TODO
// convert schedule
// /home/amir/.cargo/registry/src/github.com-1ecc6299db9ec823/clockwork-cron-1.4.0/src/schedule.rs

#[async_trait]
impl CommandTrait for Payment {
    fn name(&self) -> Name {
        CREATE_PAYMENT.into()
    }

    fn inputs(&self) -> Vec<CmdInput> {
        [
            CmdInput {
                name: IS_IMMEDIATE.into(),
                type_bounds: [ValueType::Bool].to_vec(),
                required: false,
                passthrough: false,
            },
            CmdInput {
                name: SCHEDULE.into(),
                type_bounds: [ValueType::String].to_vec(),
                required: false,
                passthrough: false,
            },
            CmdInput {
                name: IS_SKIPPABLE.into(),
                type_bounds: [ValueType::Bool].to_vec(),
                required: false,
                passthrough: false,
            },
            CmdInput {
                name: MONITOR_ACCOUNT.into(),
                type_bounds: [ValueType::Keypair, ValueType::String, ValueType::Pubkey].to_vec(),
                required: false,
                passthrough: false,
            },
            CmdInput {
                name: PAYER.into(),
                type_bounds: [ValueType::Keypair, ValueType::String].to_vec(),
                required: true,
                passthrough: true,
            },
            CmdInput {
                name: TOKEN_MINT.into(),
                type_bounds: [ValueType::Pubkey, ValueType::Keypair, ValueType::String].to_vec(),
                required: true,
                passthrough: false,
            },
            CmdInput {
                name: TOKEN_ACCOUNT.into(),
                type_bounds: [ValueType::Pubkey, ValueType::Keypair, ValueType::String].to_vec(),
                required: true,
                passthrough: false,
            },
            CmdInput {
                name: RECIPIENT.into(),
                type_bounds: [ValueType::Keypair, ValueType::String, ValueType::Pubkey].to_vec(),
                required: true,
                passthrough: false,
            },
            CmdInput {
                name: AMOUNT.into(),
                type_bounds: [ValueType::U64].to_vec(),
                required: true,
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
                name: THREAD.into(),
                r#type: ValueType::Pubkey,
            },
        ]
        .to_vec()
    }

    async fn run(&self, ctx: Context, inputs: ValueSet) -> Result<ValueSet, CommandError> {
        let trigger = match value::from_map(inputs.clone())? {
            Input::IsImmediate { is_immediate: _ } => Trigger::Now,
            Input::Schedule {
                schedule,
                is_skippable,
            } => Trigger::Cron {
                schedule,
                skippable: is_skippable,
            },
            Input::MonitorAccount { monitor_account } => {
                Trigger::Account {
                    address: monitor_account,
                    offset: 0, // FIXME
                    size: 32,  //FIXME
                }
            }
        };

        let InputStruct {
            payer,
            token_account,
            token_mint,
            recipient,
            amount,
        } = value::from_map(inputs)?;

        // Thread Authority is the Payer
        let thread_authority = payer.pubkey();

        // Derive PDAs
        // create into a command
        let payment = ClockworkPayment::pubkey(payer.pubkey(), token_mint, recipient);
        let thread = Thread::pubkey(thread_authority, "payment".into());

        // Get recipient's Associated Token Account
        let recipient_ata_pubkey = get_associated_token_address(&recipient, &token_mint);

        // Create Instructions
        let (minimum_balance_for_rent_exemption, instructions) = self
            .command_create_payment(
                &ctx.solana_client,
                payer.pubkey(),
                token_account,
                token_mint,
                payment,
                thread,
                recipient,
                recipient_ata_pubkey,
                amount,
                trigger,
            )
            .await?;

        let (mut transaction, recent_blockhash) = execute(
            &ctx.solana_client,
            &payer.pubkey(),
            &instructions,
            minimum_balance_for_rent_exemption,
        )
        .await?;

        /*
        println!(
            "thread: 🔗 {}",
            explorer().thread_url(thread, thread_program_ID)
        );
        */
        try_sign_wallet(&ctx, &mut transaction, &[&payer], recent_blockhash).await?;

        let signature = submit_transaction(&ctx.solana_client, transaction).await?;

        Ok(value::to_map(&Output { thread, signature })?)
    }
}

inventory::submit!(CommandDescription::new(CREATE_PAYMENT, |_| {
    Box::new(Payment)
}));

pub fn explorer() -> Explorer {
    #[cfg(feature = "localnet")]
    return Explorer::custom("http://localhost:8899".to_string());
    #[cfg(not(feature = "localnet"))]
    Explorer::devnet()
}

pub fn anchor_sighash(name: &str) -> [u8; 8] {
    let namespace = "global";
    let preimage = format!("{}:{}", namespace, name);
    let mut sighash = [0u8; 8];
    sighash.copy_from_slice(
        &anchor_lang::solana_program::hash::hash(preimage.as_bytes()).to_bytes()[..8],
    );
    sighash
}
