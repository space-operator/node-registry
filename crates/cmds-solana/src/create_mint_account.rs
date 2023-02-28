use crate::{
    prelude::*,
    utils::{execute, submit_transaction},
};
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
    ) -> crate::Result<(u64, Vec<solana_sdk::instruction::Instruction>)> {
        let minimum_balance_for_rent_exemption = rpc_client
            .get_minimum_balance_for_rent_exemption(Mint::LEN)
            .await?;

        let instructions = vec![
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
        ];

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
    #[serde(with = "value::pubkey::opt")]
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
    #[serde(with = "value::signature::opt")]
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
    fn name(&self) -> Name {
        SOLANA_CREATE_MINT_ACCOUNT.into()
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
                name: DECIMALS.into(),
                type_bounds: [ValueType::U64].to_vec(),
                required: true,
                passthrough: false,
            },
            CmdInput {
                name: MINT_AUTHORITY.into(),
                type_bounds: [ValueType::Keypair, ValueType::String].to_vec(),
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
                type_bounds: [ValueType::Keypair, ValueType::String].to_vec(),
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

    async fn run(&self, ctx: Context, inputs: ValueSet) -> Result<ValueSet, CommandError> {
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
            &[&input.mint_authority, &input.fee_payer, &input.mint_account],
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

inventory::submit!(CommandDescription::new(SOLANA_CREATE_MINT_ACCOUNT, |_| {
    Box::new(CreateMintAccount)
}));
