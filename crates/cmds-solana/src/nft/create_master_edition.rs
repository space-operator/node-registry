use crate::prelude::*;
use solana_program::instruction::Instruction;

#[derive(Debug, Clone)]
pub struct CreateMasterEdition;

impl CreateMasterEdition {
    #[allow(clippy::too_many_arguments)]
    async fn command_create_master_edition(
        &self,
        rpc_client: &RpcClient,
        metadata_pubkey: Pubkey,
        master_edition_pubkey: Pubkey,
        mint: Pubkey,
        mint_authority: Pubkey,
        payer: Pubkey,
        update_authority: Pubkey,
        max_supply: u64,
    ) -> crate::Result<(u64, Vec<Instruction>)> {
        let minimum_balance_for_rent_exemption = rpc_client
            .get_minimum_balance_for_rent_exemption(std::mem::size_of::<
                mpl_token_metadata::state::MasterEditionV2,
            >())
            .await?;

        let instructions = vec![mpl_token_metadata::instruction::create_master_edition_v3(
            mpl_token_metadata::id(),
            master_edition_pubkey,
            mint,
            update_authority,
            mint_authority,
            metadata_pubkey,
            payer,
            Some(max_supply),
        )];

        Ok((minimum_balance_for_rent_exemption, instructions))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Input {
    #[serde(with = "value::pubkey")]
    pub mint_account: Pubkey,
    #[serde(with = "value::pubkey")]
    pub mint_authority: Pubkey,
    #[serde(with = "value::keypair")]
    pub fee_payer: Keypair,
    #[serde(with = "value::keypair")]
    pub update_authority: Keypair,
    pub max_supply: u64,
    #[serde(default = "value::default::bool_true")]
    pub submit: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    #[serde(with = "value::signature::opt")]
    pub signature: Option<Signature>,
    #[serde(with = "value::pubkey")]
    pub metadata_account: Pubkey,
    #[serde(with = "value::pubkey")]
    pub master_edition_account: Pubkey,
}

const CREATE_MASTER_EDITION: &str = "create_master_edition";

// Inputs
const MINT_ACCOUNT: &str = "mint_account";
const MINT_AUTHORITY: &str = "mint_authority";
const FEE_PAYER: &str = "fee_payer";
const UPDATE_AUTHORITY: &str = "update_authority";
const MAX_SUPPLY: &str = "max_supply";
const SUBMIT: &str = "submit";

// Outputs
const SIGNATURE: &str = "signature";
const METADATA_ACCOUNT: &str = "metadata_account";
const MASTER_EDITION_ACCOUNT: &str = "master_edition_account";

#[async_trait]
impl CommandTrait for CreateMasterEdition {
    fn name(&self) -> Name {
        CREATE_MASTER_EDITION.into()
    }

    fn inputs(&self) -> Vec<CmdInput> {
        [
            CmdInput {
                name: MINT_ACCOUNT.into(),
                type_bounds: [ValueType::Pubkey, ValueType::Keypair, ValueType::String].to_vec(),
                required: true,
                passthrough: true,
            },
            CmdInput {
                name: MINT_AUTHORITY.into(),
                type_bounds: [ValueType::Pubkey, ValueType::Keypair, ValueType::String].to_vec(),
                required: true,
                passthrough: false,
            },
            CmdInput {
                name: FEE_PAYER.into(),
                type_bounds: [ValueType::Keypair, ValueType::String].to_vec(),
                required: true,
                passthrough: true,
            },
            CmdInput {
                name: UPDATE_AUTHORITY.into(),
                type_bounds: [ValueType::Keypair, ValueType::String].to_vec(),
                required: true,
                passthrough: false,
            },
            CmdInput {
                name: MAX_SUPPLY.into(),
                type_bounds: [ValueType::U64].to_vec(),
                required: true,
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
                name: METADATA_ACCOUNT.into(),
                r#type: ValueType::Pubkey,
            },
            CmdOutput {
                name: MASTER_EDITION_ACCOUNT.into(),
                r#type: ValueType::Pubkey,
            },
        ]
        .to_vec()
    }

    async fn run(&self, ctx: Context, inputs: ValueSet) -> Result<ValueSet, CommandError> {
        let Input {
            mint_account,
            mint_authority,
            fee_payer,
            update_authority,
            max_supply,
            submit,
        } = value::from_map(inputs)?;

        let (metadata_account, _) = mpl_token_metadata::pda::find_metadata_account(&mint_account);

        let (master_edition_account, _) =
            mpl_token_metadata::pda::find_master_edition_account(&mint_account);

        let (minimum_balance_for_rent_exemption, instructions) = self
            .command_create_master_edition(
                &ctx.solana_client,
                metadata_account,
                master_edition_account,
                mint_account,
                mint_authority,
                fee_payer.pubkey(),
                update_authority.pubkey(),
                max_supply,
            )
            .await?;

        let fee_payer_pubkey = fee_payer.pubkey();

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
            &[&update_authority, &fee_payer],
            recent_blockhash,
        )
        .await?;

        let signature = if submit {
            Some(submit_transaction(&ctx.solana_client, transaction).await?)
        } else {
            None
        };

        Ok(value::to_map(&Output {
            metadata_account,
            master_edition_account,
            signature,
        })?)
    }
}

inventory::submit!(CommandDescription::new(
    CREATE_MASTER_EDITION,
    |_| Box::new(CreateMasterEdition)
));
