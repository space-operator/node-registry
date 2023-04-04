use crate::{
    prelude::*,
    utils::{tx_to_string, ui_amount_to_amount},
};
use solana_program::system_program;
use solana_sdk::instruction::Instruction;
use solana_sdk::program_pack::Pack;
use spl_associated_token_account::instruction;
use spl_token::instruction::transfer_checked;

#[derive(Debug, Clone)]
pub struct TransferToken;

impl TransferToken {
    // https://spl.solana.com/associated-token-account
    // https://github.com/solana-labs/solana-program-library/blob/master/token/cli/src/main.rs#L555
    #[allow(clippy::too_many_arguments)]
    pub async fn command_transfer_token(
        &self,
        client: &RpcClient,
        fee_payer: &Pubkey,
        token: Pubkey,
        ui_amount: Decimal,
        recipient: Pubkey,
        sender: Option<Pubkey>,
        sender_owner: Pubkey,
        allow_unfunded_recipient: bool,
        fund_recipient: bool,
        memo: String,
    ) -> crate::Result<(u64, Vec<Instruction>, Pubkey)> {
        let sender = if let Some(sender) = sender {
            sender
        } else {
            spl_associated_token_account::get_associated_token_address(&sender_owner, &token)
        };

        let (_, decimals) = resolve_mint_info(client, sender).await?;
        let transfer_balance = {
            // TODO error handling
            let sender_token_amount = client.get_token_account_balance(&sender).await?;

            // TODO error handling
            let sender_balance = sender_token_amount
                .amount
                .parse::<u64>()
                .map_err(crate::Error::custom)?;

            let transfer_balance = ui_amount_to_amount(ui_amount, decimals)?;
            if transfer_balance > sender_balance {
                // TODO: discuss if this error appropriate for token semantically?
                return Err(crate::Error::InsufficientSolanaBalance {
                    needed: transfer_balance,
                    balance: sender_balance,
                });
            }
            transfer_balance
        };

        let mut recipient_token_account = recipient;
        let mut minimum_balance_for_rent_exemption = 0;

        let recipient_is_token_account = {
            let recipient_account_info = client
                .get_account_with_commitment(&recipient, client.commitment())
                .await?
                .value
                .map(|account| {
                    account.owner == spl_token::id()
                        && account.data.len() == spl_token::state::Account::LEN
                });

            if recipient_account_info.is_none() && !allow_unfunded_recipient {
                return Err(crate::Error::RecipientAddressNotFunded);
            }
            recipient_account_info.unwrap_or(false)
        };

        let mut instructions = vec![];
        if !recipient_is_token_account {
            recipient_token_account =
                spl_associated_token_account::get_associated_token_address(&recipient, &token);

            let needs_funding = {
                if let Some(recipient_token_account_data) = client
                    .get_account_with_commitment(&recipient_token_account, client.commitment())
                    .await?
                    .value
                {
                    match recipient_token_account_data.owner {
                        x if x == system_program::id() => true,
                        y if y == spl_token::id() => false,
                        _ => {
                            return Err(crate::Error::UnsupportedRecipientAddress(
                                recipient.to_string(),
                            ))
                        }
                    }
                } else {
                    true
                }
            };

            if needs_funding {
                if fund_recipient {
                    minimum_balance_for_rent_exemption += client
                        .get_minimum_balance_for_rent_exemption(spl_token::state::Account::LEN)
                        .await?;
                    instructions.push(instruction::create_associated_token_account(
                        fee_payer,
                        &recipient,
                        &token,
                        &spl_associated_token_account::ID,
                    ));
                } else {
                    // TODO: discuss the logic of this error
                    return Err(crate::Error::AssociatedTokenAccountDoesntExist);
                }
            }
        }

        instructions.push(transfer_checked(
            &spl_token::id(),
            &sender,
            &token,
            &recipient_token_account,
            &sender_owner,
            &[&sender_owner, fee_payer],
            transfer_balance,
            decimals,
        )?);

        instructions.push(spl_memo::build_memo(memo.as_bytes(), &[fee_payer]));

        Ok((
            minimum_balance_for_rent_exemption,
            instructions,
            recipient_token_account,
        ))
    }
}

pub(crate) async fn resolve_mint_info(
    client: &RpcClient,
    token_account: Pubkey,
) -> crate::Result<(Pubkey, u8)> {
    let source_account = client
        .get_token_account(&token_account)
        .await
        .map_err(|_| crate::Error::NotTokenAccount(token_account))?
        .ok_or(crate::Error::NotTokenAccount(token_account))?;
    let source_mint = source_account
        .mint
        .parse::<Pubkey>()
        .map_err(crate::Error::custom)?;
    Ok((source_mint, source_account.token_amount.decimals))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Input {
    #[serde(with = "value::keypair")]
    pub fee_payer: Keypair,
    #[serde(with = "value::pubkey")]
    pub mint_account: Pubkey,
    #[serde(default)]
    pub memo: String,
    #[serde(with = "value::decimal")]
    pub amount: Decimal,
    #[serde(with = "value::pubkey")]
    pub recipient: Pubkey,
    #[serde(default, with = "value::pubkey::opt")]
    pub sender_token_account: Option<Pubkey>,
    #[serde(with = "value::keypair")]
    pub sender_owner: Keypair,
    #[serde(default = "value::default::bool_true")]
    pub allow_unfunded: bool,
    #[serde(default = "value::default::bool_true")]
    pub fund_recipient: bool,
    #[serde(default = "value::default::bool_true")]
    pub submit: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    pub tx: String,
    #[serde(with = "value::pubkey")]
    pub recipient_token_account: Pubkey,
    #[serde(default, with = "value::signature::opt")]
    pub signature: Option<Signature>,
}

const SOLANA_TRANSFER_TOKEN: &str = "transfer_token";

// Inputs
const FEE_PAYER: &str = "fee_payer";
const MINT_ACCOUNT: &str = "mint_account";
const AMOUNT: &str = "amount";
const RECIPIENT: &str = "recipient";
const SENDER_TOKEN_ACCOUNT: &str = "sender_token_account";
const SENDER_OWNER: &str = "sender_owner";
const ALLOW_UNFUNDED: &str = "allow_unfunded";
const FUND_RECIPIENT: &str = "fund_recipient";
const MEMO: &str = "memo";
const SUBMIT: &str = "submit";

// Outputs
const TX: &str = "tx";
const SIGNATURE: &str = "signature";
const RECIPIENT_TOKEN_ACCOUNT: &str = "recipient_token_account";

#[async_trait]
impl CommandTrait for TransferToken {
    fn name(&self) -> Name {
        SOLANA_TRANSFER_TOKEN.into()
    }

    fn inputs(&self) -> Vec<CmdInput> {
        [
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
                passthrough: false,
            },
            CmdInput {
                name: AMOUNT.into(),
                type_bounds: [ValueType::F64].to_vec(),
                required: true,
                passthrough: false,
            },
            CmdInput {
                name: RECIPIENT.into(),
                type_bounds: [ValueType::Pubkey].to_vec(),
                required: true,
                passthrough: true,
            },
            CmdInput {
                name: SENDER_TOKEN_ACCOUNT.into(),
                type_bounds: [ValueType::Pubkey].to_vec(),
                required: false,
                passthrough: true,
            },
            CmdInput {
                name: SENDER_OWNER.into(),
                type_bounds: [ValueType::Keypair].to_vec(),
                required: true,
                passthrough: true,
            },
            CmdInput {
                name: ALLOW_UNFUNDED.into(),
                type_bounds: [ValueType::Bool].to_vec(),
                required: false,
                passthrough: false,
            },
            CmdInput {
                name: FUND_RECIPIENT.into(),
                type_bounds: [ValueType::Bool].to_vec(),
                required: false,
                passthrough: false,
            },
            CmdInput {
                name: MEMO.into(),
                type_bounds: [ValueType::String].to_vec(),
                required: false,
                passthrough: false,
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
                name: SIGNATURE.into(),
                r#type: ValueType::String,
            },
            CmdOutput {
                name: TX.into(),
                r#type: ValueType::String,
            },
            CmdOutput {
                name: RECIPIENT_TOKEN_ACCOUNT.into(),
                r#type: ValueType::Pubkey,
            },
        ]
        .to_vec()
    }

    async fn run(&self, ctx: Context, inputs: ValueSet) -> Result<ValueSet, CommandError> {
        let input: Input = value::from_map(inputs)?;

        let (minimum_balance_for_rent_exemption, instructions, recipient_token_account) = self
            .command_transfer_token(
                &ctx.solana_client,
                &input.fee_payer.pubkey(),
                input.mint_account,
                input.amount,
                input.recipient,
                input.sender_token_account,
                input.sender_owner.pubkey(),
                input.allow_unfunded,
                input.fund_recipient,
                input.memo,
            )
            .await?;

        let fee_payer_pubkey = input.fee_payer.pubkey();

        let (mut transaction, recent_blockhash) = execute(
            &ctx.solana_client,
            &fee_payer_pubkey,
            &instructions,
            minimum_balance_for_rent_exemption,
        )
        .await?;

        try_sign_wallet(
            &ctx,
            &mut transaction,
            &[&input.fee_payer, &input.sender_owner],
            recent_blockhash,
        )
        .await?;
        let tx_str = tx_to_string(&transaction)?;

        let signature = if input.submit {
            Some(submit_transaction(&ctx.solana_client, transaction).await?)
        } else {
            None
        };

        Ok(value::to_map(&Output {
            signature,
            recipient_token_account,
            tx: tx_str,
        })?)
    }
}

inventory::submit!(CommandDescription::new(SOLANA_TRANSFER_TOKEN, |_| Ok(
    Box::new(TransferToken)
)));
