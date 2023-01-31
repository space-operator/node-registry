use crate::{
    prelude::*,
    utils::{execute, submit_transaction},
};
use solana_program::program_pack::Pack;
use spl_associated_token_account::instruction::create_associated_token_account;

#[derive(Debug, Clone)]
pub struct AssociatedTokenAccount;

#[derive(Serialize, Deserialize, Debug)]
pub struct Input {
    #[serde(with = "value::pubkey")]
    owner: Pubkey,
    #[serde(with = "value::keypair")]
    fee_payer: Keypair,
    #[serde(with = "value::pubkey")]
    mint_account: Pubkey,
    #[serde(default = "value::default::bool_true")]
    submit: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    #[serde(with = "value::pubkey")]
    associated_token_account: Pubkey,
    #[serde(with = "value::signature::opt")]
    signature: Option<Signature>,
}

const SOLANA_ASSOCIATED_TOKEN_ACCOUNT: &str = "associated_token_account";

// Inputs
const OWNER: &str = "owner";
const FEE_PAYER: &str = "fee_payer";
const MINT_ACCOUNT: &str = "mint_account";
const SUBMIT: &str = "submit";

// Outputs
const ASSOCIATED_TOKEN_ACCOUNT: &str = "associated_token_account";
const SIGNATURE: &str = "signature";

#[async_trait]
impl CommandTrait for AssociatedTokenAccount {
    fn name(&self) -> Name {
        SOLANA_ASSOCIATED_TOKEN_ACCOUNT.into()
    }

    fn inputs(&self) -> Vec<CmdInput> {
        [
            CmdInput {
                name: OWNER.into(),
                type_bounds: [ValueType::Pubkey, ValueType::Keypair, ValueType::String].to_vec(),
                required: true,
                passthrough: true,
            },
            CmdInput {
                name: FEE_PAYER.into(),
                type_bounds: [ValueType::Keypair, ValueType::String].to_vec(),
                required: true,
                passthrough: true,
            },
            CmdInput {
                name: MINT_ACCOUNT.into(),
                type_bounds: [ValueType::Pubkey, ValueType::Keypair, ValueType::String].to_vec(),
                required: true,
                passthrough: true,
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
                name: ASSOCIATED_TOKEN_ACCOUNT.into(),
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
        let input: Input = value::from_map(inputs)?;

        let minimum_balance_for_rent_exemption = ctx
            .solana_client
            .get_minimum_balance_for_rent_exemption(spl_token::state::Account::LEN)
            .await?;

        let instructions = create_associated_token_account(
            &input.fee_payer.pubkey(),
            &input.owner,
            &input.mint_account,
            &spl_token::id(),
        );

        let associated_token_account = instructions.accounts[1].pubkey;

        let (mut transaction, recent_blockhash) = execute(
            &ctx.solana_client,
            &input.fee_payer.pubkey(),
            &[instructions],
            minimum_balance_for_rent_exemption,
        )
        .await?;

        try_sign_wallet(
            &ctx,
            &mut transaction,
            &[&input.fee_payer],
            recent_blockhash,
        )
        .await?;

        let signature = if input.submit {
            Some(submit_transaction(&ctx.solana_client, transaction).await?)
        } else {
            None
        };

        Ok(value::to_map(&Output {
            associated_token_account,
            signature,
        })?)
    }
}

inventory::submit!(CommandDescription::new(
    SOLANA_ASSOCIATED_TOKEN_ACCOUNT,
    |_| { Box::new(AssociatedTokenAccount) }
));
