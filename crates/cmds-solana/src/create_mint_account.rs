use crate::prelude::*;
use solana_sdk::program_pack::Pack;
use solana_sdk::system_instruction;
use spl_token::state::Mint;

#[derive(Debug, Clone)]
pub struct CreateMintAccount;

impl CreateMintAccount {
    pub async fn command_create_mint_account(
        &self,
        rpc_client: &RpcClient,
        fee_payer: &Pubkey,
        decimals: u8,
        mint_account: &Pubkey,
        mint_authority: Pubkey,
        freeze_authority: Option<Pubkey>,
        memo: &str,
    ) -> crate::Result<(u64, Vec<Instruction>)> {
        let minimum_balance_for_rent_exemption = rpc_client
            .get_minimum_balance_for_rent_exemption(Mint::LEN)
            .await?;

        let instructions = [
            system_instruction::create_account(
                fee_payer,
                mint_account,
                minimum_balance_for_rent_exemption,
                Mint::LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_mint2(
                &spl_token::id(),
                mint_account,
                &mint_authority,
                freeze_authority.as_ref(),
                decimals,
            )?,
            spl_memo::build_memo(memo.as_bytes(), &[fee_payer]),
        ]
        .to_vec();

        Ok((minimum_balance_for_rent_exemption, instructions))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Input {
    #[serde(with = "value::keypair")]
    fee_payer: Keypair,
    decimals: u8,
    #[serde(with = "value::keypair")]
    mint_authority: Keypair,
    #[serde(default, with = "value::pubkey::opt")]
    freeze_authority: Option<Pubkey>,
    #[serde(with = "value::keypair")]
    mint_account: Keypair,
    #[serde(default)]
    memo: String,
    #[serde(default = "value::default::bool_true")]
    submit: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    #[serde(default, with = "value::signature::opt")]
    signature: Option<Signature>,
}

const SOLANA_CREATE_MINT_ACCOUNT: &str = "create_mint_account";

// Inputs
const FEE_PAYER: &str = "fee_payer";
const DECIMALS: &str = "decimals";
const MINT_AUTHORITY: &str = "mint_authority";
const FREEZE_AUTHORITY: &str = "freeze_authority";
const MINT_ACCOUNT: &str = "mint_account";
const MEMO: &str = "memo";
const SUBMIT: &str = "submit";

// Outputs
const SIGNATURE: &str = "signature";

#[async_trait]
impl CommandTrait for CreateMintAccount {
    fn instruction_info(&self) -> Option<InstructionInfo> {
        Some(InstructionInfo::simple(self, SIGNATURE))
    }

    fn name(&self) -> Name {
        SOLANA_CREATE_MINT_ACCOUNT.into()
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
                name: DECIMALS.into(),
                type_bounds: [ValueType::U8].to_vec(),
                required: true,
                passthrough: false,
            },
            CmdInput {
                name: MINT_AUTHORITY.into(),
                type_bounds: [ValueType::Keypair].to_vec(),
                required: true,
                passthrough: true,
            },
            CmdInput {
                name: FREEZE_AUTHORITY.into(),
                type_bounds: [ValueType::Keypair, ValueType::String, ValueType::Pubkey].to_vec(),
                required: false,
                passthrough: true,
            },
            CmdInput {
                name: MINT_ACCOUNT.into(),
                type_bounds: [ValueType::Keypair].to_vec(),
                required: true,
                passthrough: true,
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
        [CmdOutput {
            name: SIGNATURE.into(),
            r#type: ValueType::String,
        }]
        .to_vec()
    }

    async fn run(&self, mut ctx: Context, inputs: ValueSet) -> Result<ValueSet, CommandError> {
        let input: Input = value::from_map(inputs)?;

        let (minimum_balance_for_rent_exemption, instructions) = self
            .command_create_mint_account(
                &ctx.solana_client,
                &input.fee_payer.pubkey(),
                input.decimals,
                &input.mint_account.pubkey(),
                input.mint_authority.pubkey(),
                input.freeze_authority,
                &input.memo,
            )
            .await?;

        let instructions = if input.submit {
            Instructions {
                fee_payer: input.fee_payer.pubkey(),
                signers: vec![
                    input.mint_authority.clone_keypair(),
                    input.fee_payer.clone_keypair(),
                    input.mint_account.clone_keypair(),
                ],
                minimum_balance_for_rent_exemption,
                instructions,
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

inventory::submit!(CommandDescription::new(SOLANA_CREATE_MINT_ACCOUNT, |_| {
    Ok(Box::new(CreateMintAccount))
}));
