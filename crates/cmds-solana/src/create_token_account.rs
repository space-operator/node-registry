use crate::{
    prelude::*,
    utils::{execute, submit_transaction},
};
use solana_program::instruction::Instruction;
use solana_program::program_pack::Pack;
use solana_program::{system_instruction, system_program};

#[derive(Debug, Clone)]
pub struct CreateTokenAccount;

impl CreateTokenAccount {
    async fn command_create_token_account(
        &self,
        client: &RpcClient,
        fee_payer: Pubkey,
        token: Pubkey,
        owner: Pubkey,
        account_pubkey: Pubkey,
    ) -> crate::Result<(u64, Vec<Instruction>)> {
        let minimum_balance_for_rent_exemption = client
            .get_minimum_balance_for_rent_exemption(spl_token::state::Account::LEN)
            .await?;

        let (account, system_account_ok, instructions) = {
            (
                account_pubkey,
                false,
                vec![
                    system_instruction::create_account(
                        &fee_payer,
                        &account_pubkey,
                        minimum_balance_for_rent_exemption,
                        spl_token::state::Account::LEN as u64,
                        &spl_token::id(),
                    ),
                    spl_token::instruction::initialize_account(
                        &spl_token::id(),
                        &account_pubkey,
                        &token,
                        &owner,
                    )?,
                ],
            )
        };

        if let Some(account_data) = client
            .get_account_with_commitment(&account, client.commitment())
            .await?
            .value
        {
            if !(account_data.owner == system_program::id() && system_account_ok) {
                return Err(crate::Error::custom(anyhow::anyhow!(
                    "Error: Account already exists: {}",
                    account
                )));
            }
        }

        Ok((minimum_balance_for_rent_exemption, instructions))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Input {
    #[serde(with = "value::pubkey")]
    owner: Pubkey,
    #[serde(with = "value::keypair")]
    fee_payer: Keypair,
    #[serde(with = "value::pubkey")]
    mint_account: Pubkey,
    #[serde(with = "value::keypair")]
    token_account: Keypair,
    #[serde(default = "value::default::bool_true")]
    submit: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    #[serde(default, with = "value::signature::opt")]
    signature: Option<Signature>,
}

const SOLANA_CREATE_TOKEN_ACCOUNT: &str = "create_token_account";

// Inputs
const OWNER: &str = "owner";
const FEE_PAYER: &str = "fee_payer";
const MINT_ACCOUNT: &str = "mint_account";
const TOKEN_ACCOUNT: &str = "token_account";
const SUBMIT: &str = "submit";

// Outputs
const SIGNATURE: &str = "signature";

#[async_trait]
impl CommandTrait for CreateTokenAccount {
    fn name(&self) -> Name {
        SOLANA_CREATE_TOKEN_ACCOUNT.into()
    }

    fn inputs(&self) -> Vec<CmdInput> {
        [
            CmdInput {
                name: OWNER.into(),
                type_bounds: [ValueType::Pubkey].to_vec(),
                required: true,
                passthrough: true,
            },
            CmdInput {
                name: FEE_PAYER.into(),
                type_bounds: [ValueType::Keypair].to_vec(),
                required: true,
                passthrough: true,
            },
            CmdInput {
                name: MINT_ACCOUNT.into(),
                type_bounds: [ValueType::Pubkey].to_vec(),
                required: true,
                passthrough: true,
            },
            CmdInput {
                name: TOKEN_ACCOUNT.into(),
                type_bounds: [ValueType::Keypair].to_vec(),
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
        [CmdOutput {
            name: SIGNATURE.into(),
            r#type: ValueType::String,
        }]
        .to_vec()
    }

    async fn run(&self, ctx: Context, inputs: ValueSet) -> Result<ValueSet, CommandError> {
        let input: Input = value::from_map(inputs)?;

        let (minimum_balance_for_rent_exemption, instructions) = self
            .command_create_token_account(
                &ctx.solana_client,
                input.fee_payer.pubkey(),
                input.mint_account,
                input.owner,
                input.token_account.pubkey(),
            )
            .await?;

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
            &[&input.fee_payer, &input.token_account],
            recent_blockhash,
        )
        .await?;

        let signature = if input.submit {
            Some(submit_transaction(&ctx.solana_client, transaction).await?)
        } else {
            None
        };

        Ok(value::to_map(&Output { signature })?)
    }
}

inventory::submit!(CommandDescription::new(SOLANA_CREATE_TOKEN_ACCOUNT, |_| {
    Box::new(CreateTokenAccount)
}));

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_invalid() {
        let input = Input {
            owner: Pubkey::new_unique(),
            fee_payer: Keypair::new(),
            mint_account: Pubkey::new_unique(),
            token_account: Keypair::new(),
            submit: false,
        };
        let input = value::to_map(&input).unwrap();

        let error = CreateTokenAccount
            .run(Context::default(), input)
            .await
            .unwrap_err();
        dbg!(error);
    }
}
