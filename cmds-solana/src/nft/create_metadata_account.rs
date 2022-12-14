use super::{NftCreator, NftMetadata, NftUses};
use crate::prelude::*;
use mpl_token_metadata::state::{Collection, CollectionDetails, Creator, Uses};
use solana_program::instruction::Instruction;

#[derive(Debug)]
pub struct CreateMetadataAccount;

impl CreateMetadataAccount {
    #[allow(clippy::too_many_arguments)]
    async fn command_create_metadata_accounts(
        &self,
        rpc_client: &RpcClient,
        metadata_pubkey: Pubkey,
        mint: Pubkey,
        mint_authority: Pubkey,
        payer: Pubkey,
        update_authority: Pubkey,
        name: String,
        symbol: String,
        uri: String,
        creators: Option<Vec<Creator>>,
        seller_fee_basis_points: u16,
        update_authority_is_signer: bool,
        is_mutable: bool,
        collection: Option<Collection>,
        uses: Uses,
        collection_details: Option<CollectionDetails>,
    ) -> crate::Result<(u64, Vec<Instruction>)> {
        let minimum_balance_for_rent_exemption = rpc_client
            .get_minimum_balance_for_rent_exemption(std::mem::size_of::<
                mpl_token_metadata::state::Metadata,
            >())
            .await?;

        let instructions = vec![
            mpl_token_metadata::instruction::create_metadata_accounts_v3(
                mpl_token_metadata::id(),
                metadata_pubkey,
                mint,
                mint_authority,
                payer,
                update_authority,
                name,
                symbol,
                uri,
                creators,
                seller_fee_basis_points,
                update_authority_is_signer,
                is_mutable,
                collection,
                Some(uses),
                collection_details,
            ),
        ];

        Ok((minimum_balance_for_rent_exemption, instructions))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Input {
    pub is_mutable: bool,
    #[serde(with = "value::pubkey")]
    pub mint_account: Pubkey,
    #[serde(with = "value::pubkey")]
    pub mint_authority: Pubkey,
    #[serde(with = "value::keypair")]
    pub fee_payer: Keypair,
    #[serde(with = "value::keypair")]
    pub update_authority: Keypair,
    pub metadata: NftMetadata,
    pub metadata_uri: String,
    pub uses: NftUses,
    #[serde(with = "value::pubkey")]
    pub collection_mint_account: Pubkey,
    pub creators: Vec<NftCreator>,
    pub collection_details: Option<u64>,
    #[serde(default = "value::default::bool_true")]
    pub submit: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    #[serde(with = "value::signature::opt")]
    signature: Option<Signature>,
    #[serde(with = "value::pubkey")]
    metadata_account: Pubkey,
}

const CREATE_METADATA_ACCOUNT: &str = "create_metadata_account";

// Inputs
const IS_MUTABLE: &str = "is_mutable";
const MINT_ACCOUNT: &str = "mint_account";
const MINT_AUTHORITY: &str = "mint_authority";
const FEE_PAYER: &str = "fee_payer";
const UPDATE_AUTHORITY: &str = "update_authority";
const METADATA: &str = "metadata";
const METADATA_URI: &str = "metadata_uri";
const USES: &str = "uses";
const COLLECTION_MINT_ACCOUNT: &str = "collection_mint_account";
const CREATORS: &str = "creators";
const COLLECTION_DETAILS: &str = "collection_details";
const SUBMIT: &str = "submit";

// Outputs
const SIGNATURE: &str = "signature";
const METADATA_ACCOUNT: &str = "metadata_account";

#[async_trait]
impl CommandTrait for CreateMetadataAccount {
    fn name(&self) -> Name {
        CREATE_METADATA_ACCOUNT.into()
    }

    fn inputs(&self) -> Vec<CmdInput> {
        [
            CmdInput {
                name: IS_MUTABLE.into(),
                type_bounds: [ValueType::Bool].to_vec(),
                required: true,
                passthrough: true,
            },
            CmdInput {
                name: MINT_ACCOUNT.into(),
                type_bounds: [ValueType::Pubkey, ValueType::Keypair, ValueType::String].to_vec(),
                required: true,
                passthrough: false,
            },
            CmdInput {
                name: MINT_AUTHORITY.into(),
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
                name: UPDATE_AUTHORITY.into(),
                type_bounds: [ValueType::Keypair, ValueType::String].to_vec(),
                required: true,
                passthrough: false,
            },
            CmdInput {
                name: METADATA.into(),
                type_bounds: [ValueType::Free].to_vec(),
                required: true,
                passthrough: false,
            },
            CmdInput {
                name: METADATA_URI.into(),
                type_bounds: [ValueType::String].to_vec(),
                required: true,
                passthrough: false,
            },
            CmdInput {
                name: USES.into(),
                type_bounds: [ValueType::Free].to_vec(),
                required: true,
                passthrough: false,
            },
            CmdInput {
                name: COLLECTION_MINT_ACCOUNT.into(),
                type_bounds: [ValueType::Pubkey, ValueType::Keypair, ValueType::String].to_vec(),
                required: true,
                passthrough: true,
            },
            CmdInput {
                name: COLLECTION_DETAILS.into(),
                type_bounds: [ValueType::U64].to_vec(),
                required: false,
                passthrough: false,
            },
            CmdInput {
                name: CREATORS.into(),
                type_bounds: [ValueType::Free].to_vec(),
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
        ]
        .to_vec()
    }

    async fn run(&self, ctx: Context, inputs: ValueSet) -> Result<ValueSet, CommandError> {
        let Input {
            is_mutable,
            mint_account,
            mint_authority,
            fee_payer,
            update_authority,
            metadata,
            metadata_uri,
            uses,
            collection_mint_account,
            creators,
            collection_details,
            submit,
        } = value::from_map(inputs)?;
        let collection_details = collection_details.map(|size| CollectionDetails::V1 { size });

        let (metadata_account, _) = mpl_token_metadata::pda::find_metadata_account(&mint_account);

        let collection = Some(Collection {
            verified: false,
            key: collection_mint_account,
        });
        let creators: Vec<Creator> = creators.into_iter().map(Creator::from).collect();
        let (minimum_balance_for_rent_exemption, instructions) = self
            .command_create_metadata_accounts(
                &ctx.solana_client,
                metadata_account,
                mint_account,
                mint_authority,
                fee_payer.pubkey(),
                update_authority.pubkey(),
                metadata.name.clone(),
                metadata.symbol.clone(),
                metadata_uri,
                Some(creators),
                metadata.seller_fee_basis_points,
                true,
                is_mutable,
                collection,
                Uses::from(uses),
                collection_details,
            )
            .await?;

        let (mut transaction, recent_blockhash) = execute(
            &ctx.solana_client,
            &fee_payer.pubkey(),
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
            signature,
        })?)
    }
}

inventory::submit!(CommandDescription::new(CREATE_METADATA_ACCOUNT, |_| {
    Box::new(CreateMetadataAccount)
}));
