use super::Trigger;
use crate::prelude::*;

use clockwork_client::thread::instruction::thread_create;
use clockwork_client::thread::state::Thread;
use clockwork_utils::thread::SerializableInstruction as ClockWorkInstruction;
use clockwork_utils::thread::Trigger as ClockWorkTrigger;

use solana_program::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;

// Command Name
const THREAD_CREATE: &str = "thread_create";

#[derive(Debug)]
pub struct ThreadCreate;

#[derive(Serialize, Deserialize, Debug)]
pub struct Input {
    pub amount: u64,
    #[serde(with = "value::keypair")]
    pub thread_authority: Keypair,
    pub id: String,
    pub instructions: Vec<Instruction>,
    #[serde(with = "value::keypair")]
    pub payer: Keypair,
    pub trigger: Trigger,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    #[serde(with = "value::pubkey")]
    thread: Pubkey,
    #[serde(with = "value::signature")]
    signature: Signature,
}

#[async_trait]
impl CommandTrait for ThreadCreate {
    fn name(&self) -> Name {
        THREAD_CREATE.into()
    }

    fn inputs(&self) -> Vec<CmdInput> {
        [
            CmdInput {
                name: "THREAD_AUTHORITY".into(),
                type_bounds: [ValueType::Keypair, ValueType::String].to_vec(),
                required: true,
                passthrough: true,
            },
            CmdInput {
                name: "ID".into(),
                type_bounds: [ValueType::String].to_vec(),
                required: true,
                passthrough: false,
            },
            CmdInput {
                name: "INSTRUCTIONS".into(),
                type_bounds: [ValueType::Free].to_vec(),
                required: true,
                passthrough: false,
            },
            CmdInput {
                name: "PAYER".into(),
                type_bounds: [ValueType::Keypair, ValueType::String].to_vec(),
                required: true,
                passthrough: false,
            },
            CmdInput {
                name: "AMOUNT".into(),
                type_bounds: [ValueType::U64].to_vec(),
                required: true,
                passthrough: false,
            },
            CmdInput {
                name: "TRIGGER".into(),
                type_bounds: [ValueType::Free].to_vec(),
                required: true,
                passthrough: false,
            },
        ]
        .to_vec()
    }

    fn outputs(&self) -> Vec<CmdOutput> {
        [
            CmdOutput {
                name: "THREAD".into(),
                r#type: ValueType::String,
            },
            CmdOutput {
                name: "SIGNATURE".into(),
                r#type: ValueType::String,
            },
        ]
        .to_vec()
    }

    async fn run(&self, ctx: Context, inputs: ValueSet) -> Result<ValueSet, CommandError> {
        let Input {
            amount,
            thread_authority,
            id,
            instructions,
            payer,
            trigger,
        } = value::from_map(inputs.clone())?;

        // FIXME
        let minimum_balance_for_rent_exemption = ctx
            .solana_client
            .get_minimum_balance_for_rent_exemption(std::mem::size_of::<
                clockwork_thread_program::accounts::ThreadCreate,
            >())
            .await?;

        // Instruction to ClockWork SerializableInstruction
        let mut instruction_chain = vec![];
        for instruction in instructions {
            let instruction = ClockWorkInstruction::from(instruction);
            instruction_chain.push(instruction);
        }

        // Trigger to ClockWork Trigger
        let trigger = ClockWorkTrigger::from(trigger);

        let id = id.as_bytes().to_vec();

        let thread = Thread::pubkey(thread_authority.pubkey(), id.clone());

        // Create Instructions
        let instructions = vec![thread_create(
            minimum_balance_for_rent_exemption + amount,
            thread_authority.pubkey(),
            id.clone(),
            instruction_chain,
            payer.pubkey(),
            thread,
            trigger,
        )];

        //
        let (mut transaction, recent_blockhash) = execute(
            &ctx.solana_client,
            &payer.pubkey(),
            &instructions,
            minimum_balance_for_rent_exemption,
        )
        .await?;

        try_sign_wallet(
            &ctx,
            &mut transaction,
            &[&payer, &thread_authority],
            recent_blockhash,
        )
        .await?;

        let signature = submit_transaction(&ctx.solana_client, transaction).await?;

        Ok(value::to_map(&Output { signature, thread })?)
    }
}

inventory::submit!(CommandDescription::new(THREAD_CREATE, |_| {
    Ok(Box::new(ThreadCreate))
}));
