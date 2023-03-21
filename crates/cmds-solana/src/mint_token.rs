use crate::{
    prelude::*,
    utils::{execute, submit_transaction, ui_amount_to_amount},
};
use solana_program::instruction::Instruction;
use solana_sdk::transaction::Transaction;
use spl_token::instruction::mint_to_checked;

#[derive(Debug)]
pub struct SolanaMintToken;

impl SolanaMintToken {
    async fn resolve_mint_info(
        &self,
        client: &RpcClient,
        token_account: Pubkey,
    ) -> crate::Result<(Pubkey, u8)> {
        let source_account = client
            .get_token_account(&token_account)
            .await
            .map_err(|_| crate::Error::NotTokenAccount(token_account))?
            .ok_or(crate::Error::NotTokenAccount(token_account))?;
        // TODO: error instead of unwrap
        let source_mint: Pubkey = source_account.mint.parse().unwrap();
        Ok((source_mint, source_account.token_amount.decimals))
    }

    async fn command_mint(
        &self,
        client: &RpcClient,
        mint_account: Pubkey,
        fee_payer: Pubkey,
        ui_amount: Decimal,
        recipient: Pubkey,
        mint_authority: Pubkey,
    ) -> crate::Result<(u64, Vec<Instruction>)> {
        let (_, decimals) = self.resolve_mint_info(client, recipient).await?;
        let amount = ui_amount_to_amount(ui_amount, decimals)?;

        let instructions = vec![mint_to_checked(
            &spl_token::id(),
            &mint_account,
            &recipient,
            &mint_authority,
            &[&fee_payer, &mint_authority],
            amount,
            decimals,
        )?];

        Ok((0, instructions))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Input {
    #[serde(with = "value::keypair")]
    fee_payer: Keypair,
    #[serde(with = "value::keypair")]
    mint_authority: Keypair,
    #[serde(with = "value::pubkey")]
    mint_account: Pubkey,
    #[serde(with = "value::pubkey")]
    recipient: Pubkey,
    #[serde(with = "value::decimal")]
    amount: Decimal,
    #[serde(default = "value::default::bool_true")]
    submit: bool,
    generate_instructions: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    #[serde(default, with = "value::signature::opt")]
    signature: Option<Signature>,
    instructions: Option<Vec<Instruction>>,
}

const SOLANA_MINT_TOKEN: &str = "mint_token";

// Inputs
const FEE_PAYER: &str = "fee_payer";
const MINT_AUTHORITY: &str = "mint_authority";
const MINT_ACCOUNT: &str = "mint_account";
const RECIPIENT: &str = "recipient";
const AMOUNT: &str = "amount";
const SUBMIT: &str = "submit";

// Outputs
const SIGNATURE: &str = "signature";

#[async_trait]
impl CommandTrait for SolanaMintToken {
    fn instruction_info(&self) -> Option<InstructionInfo> {
        Some(InstructionInfo::simple(self, SIGNATURE))
    }

    fn name(&self) -> Name {
        SOLANA_MINT_TOKEN.into()
    }

    fn inputs(&self) -> Vec<CmdInput> {
        [
            CmdInput {
                name: FEE_PAYER.into(),
                type_bounds: [ValueType::Keypair, ValueType::String].to_vec(),
                required: true,
                passthrough: true,
            },
            CmdInput {
                name: MINT_AUTHORITY.into(),
                type_bounds: [ValueType::Keypair, ValueType::String].to_vec(),
                required: true,
                passthrough: false,
            },
            CmdInput {
                name: MINT_ACCOUNT.into(),
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
            CmdInput {
                name: "generate_instructions".into(),
                type_bounds: [ValueType::Bool].to_vec(),
                required: false, // TODO: should this be required?
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
                name: "instructions".into(),
                // We can add Instruction as a new variant to the ValueType enum
                // or we could just use Json or a String to represent the encoded instructions.
                r#type: ValueType::Json,
            },
        ]
        .to_vec()
    }

    async fn run(&self, ctx: Context, inputs: ValueSet) -> Result<ValueSet, CommandError> {
        let input: Input = value::from_map(inputs)?;

        let (minimum_balance_for_rent_exemption, instructions) = self
            .command_mint(
                &ctx.solana_client,
                input.mint_account,
                input.fee_payer.pubkey(),
                input.amount,
                input.recipient,
                input.mint_authority.pubkey(),
            )
            .await?;

        if input.generate_instructions {
            return Ok(value::to_map(&Output {
                signature: None,
                instructions: Some(instructions),
            })?);
        }

        // BUNDLING
        //
        //
        // Option 1 - add execute as command
        // - We keep what have and add the utils/execute() as a command
        // - User deciding to bundle transactions changes instruction input to true on commands and
        //   connects the output(instruction, signers?) to the execute input
        //   which bundles the instructions into one transaction for the user to sign.
        //
        // Option 2 - add execute as command with auto-execute
        // - We allow commands to auto-execute without having to add the execute command/connect all the edges.
        // - We still add execute() as a command as it is really useful to have for users to bundle and run instructions
        //
        //
        //
        //
        // Implementation
        //
        // 1. output instructions in the commands
        //
        // 2. collect instructions from nodes that created instructions and collect the list of required signers,
        //    somewhere after the flow graph is constructed and all of the commands have run,
        //
        // 3. have a way of knowing if we need to dispatch a transaction after the flow has run.
        //    maybe we could add a new node to the graph and run that.
        //
        // 4. batch the instructions into as few transactions as possible (1232 bytes per tx), ideally only one
        //  https://solana.wiki/docs/solidity-guide/transactions/#:~:text=The%20entire%20encoded%20size%20of%20a%20Solana%20transaction%20cannot%20exceed%201232%20bytes.
        //
        // If 1232 bytes are not enough, we can later upgrade to Solana new transaction version
        // TxV0/TxV1 to batch instructions into a single transaction,
        // using the solana Look Up Table Program.
        //
        // client.get_recent_blockhash();
        // let tx = Transaction::new(&[signers], &[instructions], recent_blockhash);
        // let serialized = tx.s
        //
        // 5. For later. Validation will be required to prevent tampering
        // - we send transaction unsigned to user
        // - we receive signed transaction, serialize, and check that message has not be tampered with
        // - we partial sign and submit

        // PDA
        //
        // update_an_nft(which_nft, authority) {
        //   is authority amir?
        //   if no { abort! }
        //
        //   pda_signer = &[seed, seed2, persons_pubkey];
        //   update_nft(which_nft, pda_signer_seeds)
        // }
        //

        // new_program(args, outside_signers, persons_pubkey) {
        //   pda_signer = &[seed, seed2, persons_pubkey];
        //   let signers = [...outside_signers, pda_signer];
        //   token_metadata_program(args, [signers])
        // }
        let (mut transaction, recent_blockhash) = execute(
            &ctx.solana_client,
            &input.fee_payer.pubkey(),
            &instructions,
            minimum_balance_for_rent_exemption,
        )
        .await?;

        try_sign_wallet(
            &ctx,
            &mut transaction,
            &[&input.fee_payer, &input.mint_authority],
            recent_blockhash,
        )
        .await?;

        let signature = if input.submit {
            Some(submit_transaction(&ctx.solana_client, transaction).await?)
        } else {
            None
        };

        Ok(value::to_map(&Output {
            signature,
            instructions: None,
        })?)
    }
}

inventory::submit!(CommandDescription::new(SOLANA_MINT_TOKEN, |_| Box::new(
    SolanaMintToken
)));
