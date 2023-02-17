use crate::{prelude::*, utils::anchor_sighash};
use solana_program::instruction::{AccountMeta, Instruction};
use solana_sdk::pubkey::Pubkey;

use clockwork_client::thread::state::Thread;

#[derive(Debug)]
pub struct ThreadDelete;

pub fn thread_delete(authority: Pubkey, thread: Pubkey, close_to: Pubkey) -> Instruction {
    Instruction {
        program_id: clockwork_thread_program::ID,
        accounts: vec![
            AccountMeta::new_readonly(authority, true),
            AccountMeta::new(close_to, false),
            AccountMeta::new(thread, false),
        ],
        data: anchor_sighash("thread_delete").to_vec(),
    }
}

impl ThreadDelete {
    #[allow(clippy::too_many_arguments)]
    async fn command_thread_delete(
        &self,
        rpc_client: &RpcClient,
        payer: Pubkey,
        thread: Pubkey,
        close_to: Pubkey,
    ) -> crate::Result<(u64, Vec<Instruction>)> {
        // FIXME min rent
        let minimum_balance_for_rent_exemption = rpc_client
            .get_minimum_balance_for_rent_exemption(80)
            .await?;

        let instructions = vec![thread_delete(payer, thread, close_to)];

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
const THREAD_DELETE: &str = "thread_delete";

// Inputs
const THREAD_AUTHORITY: &str = "thread_authority";
const PAYER: &str = "payer";
const THREAD: &str = "thread";
const CLOSE_TO: &str = "close_to";

// Outputs
const SIGNATURE: &str = "signature";

#[async_trait]
impl CommandTrait for ThreadDelete {
    fn name(&self) -> Name {
        THREAD_DELETE.into()
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

        let thread_input: Pubkey =
            thread.unwrap_or_else(|| Thread::pubkey(thread_authority.pubkey(), "payment".into()));

        let close_to_input: Pubkey = close_to.unwrap_or(thread_authority.pubkey());

        // Create Instructions
        let (minimum_balance_for_rent_exemption, instructions) = self
            .command_thread_delete(
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

inventory::submit!(CommandDescription::new(THREAD_DELETE, |_| {
    Box::new(ThreadDelete)
}));
