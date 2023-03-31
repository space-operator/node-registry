use crate::{prelude::*, utils::anchor_sighash};
use clockwork_client::thread::{instruction::thread_create, state::Thread};
use clockwork_thread_program::state::{InstructionData, Trigger};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    system_program,
};
use solana_sdk::pubkey::Pubkey;

#[derive(Debug)]
pub struct ThreadCreate;

impl ThreadCreate {
    #[allow(clippy::too_many_arguments)]
    async fn command_thread_create(
        &self,
        rpc_client: &RpcClient,
        authority: Pubkey,
        id: String,
        payer: Pubkey,
        thread: Pubkey,
        trigger: Trigger,
        instructions: Vec<InstructionData>,
    ) -> crate::Result<(u64, Vec<Instruction>)> {
        // FIXME min rent
        let minimum_balance_for_rent_exemption = rpc_client
            .get_minimum_balance_for_rent_exemption(80)
            .await?;

        let instructions = vec![thread_create(
            authority,
            id,
            instructions,
            payer,
            thread,
            trigger,
        )];

        Ok((minimum_balance_for_rent_exemption, instructions))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Input {
    #[serde(with = "value::keypair")]
    pub thread_authority: Keypair,
    #[serde(default, with = "value::keypair::opt")]
    pub payer: Option<Keypair>,
    #[serde(default, with = "value::pubkey::opt")]
    pub thread: Option<Pubkey>,
    #[serde(default, with = "value::pubkey::opt")]
    pub close_to: Option<Pubkey>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    #[serde(with = "value::signature")]
    signature: Signature,
}

// Command Name
const THREAD_CREATE: &str = "thread_create";

// Inputs
const THREAD_AUTHORITY: &str = "thread_authority";
const PAYER: &str = "payer";
const THREAD: &str = "thread";
const CLOSE_TO: &str = "close_to";

// Outputs
const SIGNATURE: &str = "signature";

#[async_trait]
impl CommandTrait for ThreadCreate {
    fn name(&self) -> Name {
        THREAD_CREATE.into()
    }

    fn inputs(&self) -> Vec<CmdInput> {
        [
            CmdInput {
                name: THREAD_AUTHORITY.into(),
                type_bounds: [ValueType::Pubkey, ValueType::Keypair, ValueType::String].to_vec(),
                required: true,
                passthrough: true,
            },
            CmdInput {
                name: PAYER.into(),
                type_bounds: [ValueType::Keypair, ValueType::String].to_vec(),
                required: false,
                passthrough: false,
            },
            CmdInput {
                name: THREAD.into(),
                type_bounds: [ValueType::Pubkey, ValueType::Keypair, ValueType::String].to_vec(),
                required: false,
                passthrough: false,
            },
            CmdInput {
                name: CLOSE_TO.into(),
                type_bounds: [ValueType::Pubkey, ValueType::Keypair, ValueType::String].to_vec(),
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
        let Input {
            thread_authority,
            payer,
            thread,
            close_to,
        } = value::from_map(inputs.clone())?;

        // Get Inputs or pass thread_authority as default
        let payer_input: Keypair = payer.unwrap_or(thread_authority.clone_keypair());

        let thread_input: Pubkey = thread.unwrap_or_else(|| {
            let thread = Thread::pubkey(thread_authority.pubkey(), "payment".into());
            thread
        });

        let close_to_input: Pubkey = close_to.unwrap_or(thread_authority.pubkey());

        // Create Instructions
        let (minimum_balance_for_rent_exemption, instructions) = self
            .command_thread_create(
                &ctx.solana_client,
                payer_input.pubkey(),
                thread_input,
                close_to_input,
            )
            .await?;

        let (mut transaction, recent_blockhash) = execute(
            &ctx.solana_client,
            &payer_input.pubkey(),
            &instructions,
            minimum_balance_for_rent_exemption,
        )
        .await?;

        try_sign_wallet(&ctx, &mut transaction, &[&payer_input], recent_blockhash).await?;

        let signature = submit_transaction(&ctx.solana_client, transaction).await?;

        Ok(value::to_map(&Output { signature })?)
    }
}

inventory::submit!(CommandDescription::new(THREAD_CREATE, |_| {
    Box::new(ThreadCreate)
}));
