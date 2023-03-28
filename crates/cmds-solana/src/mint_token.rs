use crate::{prelude::*, utils::ui_amount_to_amount};
use solana_program::instruction::Instruction;
use spl_token::instruction::mint_to_checked;

#[derive(Debug)]
pub struct SolanaMintToken;

impl SolanaMintToken {
    async fn get_decimals(&self, client: &RpcClient, token_account: Pubkey) -> crate::Result<u8> {
        let source_account = client
            .get_token_account(&token_account)
            .await
            .map_err(|_| crate::Error::NotTokenAccount(token_account))?
            .ok_or(crate::Error::NotTokenAccount(token_account))?;
        Ok(source_account.token_amount.decimals)
    }

    async fn command_mint(
        &self,
        client: &RpcClient,
        mint_account: Pubkey,
        fee_payer: Pubkey,
        ui_amount: Decimal,
        recipient: Pubkey,
        mint_authority: Pubkey,
        decimals: Option<u8>,
    ) -> crate::Result<(u64, Vec<Instruction>)> {
        let decimals = match decimals {
            Some(d) => d,
            None => self.get_decimals(client, recipient).await?,
        };
        let amount = ui_amount_to_amount(ui_amount, decimals)?;

        let instructions = [mint_to_checked(
            &spl_token::id(),
            &mint_account,
            &recipient,
            &mint_authority,
            &[&fee_payer, &mint_authority],
            amount,
            decimals,
        )?]
        .to_vec();

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
    decimals: Option<u8>,
    #[serde(default = "value::default::bool_true")]
    submit: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    #[serde(default, with = "value::signature::opt")]
    signature: Option<Signature>,
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
                name: "decimals".into(),
                type_bounds: [ValueType::U8].to_vec(),
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
        [CmdOutput {
            name: SIGNATURE.into(),
            r#type: ValueType::String,
        }]
        .to_vec()
    }

    async fn run(&self, mut ctx: Context, inputs: ValueSet) -> Result<ValueSet, CommandError> {
        let input: Input = value::from_map(inputs)?;

        let (minimum_balance_for_rent_exemption, instructions) = self
            .command_mint(
                &ctx.solana_client,
                input.mint_account,
                input.fee_payer.pubkey(),
                input.amount,
                input.recipient,
                input.mint_authority.pubkey(),
                input.decimals,
            )
            .await?;

        let instructions = if input.submit {
            Instructions {
                fee_payer: input.fee_payer.pubkey(),
                signers: vec![
                    input.fee_payer.clone_keypair(),
                    input.mint_authority.clone_keypair(),
                ],
                instructions,
                minimum_balance_for_rent_exemption,
            }
        } else {
            Instructions::default()
        };

        let signature = ctx
            .execute(instructions, Default::default())
            .await?
            .signature;

        Ok(value::to_map(&Output { signature })?)
    }
}

inventory::submit!(CommandDescription::new(SOLANA_MINT_TOKEN, |_| Box::new(
    SolanaMintToken
)));
