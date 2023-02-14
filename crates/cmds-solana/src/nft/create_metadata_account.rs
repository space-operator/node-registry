use super::{NftCreator, NftMetadata, NftUses};
use crate::{prelude::*, proxy_authority::utils::find_proxy_authority_address};
use anchor_lang::Discriminator;
use borsh::BorshSerialize;
use mpl_token_metadata::state::{Collection, CollectionDetails, Creator, Uses};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    system_program,
};
use solana_sdk::{pubkey, pubkey::Pubkey};
use space_wrapper::instruction::ProxyCreateMetadataV3 as Proxy;

#[derive(Debug)]
pub struct CreateMetadataAccount;

pub fn create_proxy_create_metadata_instruction(
    proxy_authority: &Pubkey,
    metadata: &Pubkey,
    mint: &Pubkey,
    mint_authority: &Pubkey,
    authority: &Pubkey,
    name: String,
    symbol: String,
    uri: String,
    creators: Vec<Creator>,
    seller_fee_basis_points: u16,
    collection: Option<Pubkey>,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*proxy_authority, false),
        AccountMeta::new(*metadata, false),
        AccountMeta::new_readonly(*mint, false),
        AccountMeta::new(*mint_authority, true),
        AccountMeta::new(*authority, true),
        AccountMeta::new_readonly(mpl_token_metadata::id(), false),
        AccountMeta::new_readonly(system_program::ID, false),
    ];

    let mut data = vec![
        name.as_bytes().to_vec(),
        symbol.as_bytes().to_vec(),
        uri.as_bytes().to_vec(),
    ];

    for creator in creators.clone() {
        let serialized = creator.try_to_vec().expect("should serialize");
        let serialized_slice = serialized.to_owned();
        data.push(serialized_slice);
    }

    let seller_fee_basis_points_bytes = seller_fee_basis_points.to_le_bytes().to_vec();
    data.push(seller_fee_basis_points_bytes);

    let collection_bytes = collection.try_to_vec().expect("should serialize");
    let collection_bytes = collection_bytes.to_vec();
    data.push(collection_bytes);

    // Create method signature hash
    let proxy = Proxy {
        name,
        symbol,
        uri,
        creators: vec![space_wrapper::state::creator::Creator {
            key: creators[0].address,
            verified: creators[0].verified,
            share: creators[0].share,
        }],
        seller_fee_basis_points,
        collection,
    };

    let mut instruction_data: Vec<u8> = Proxy::discriminator().try_to_vec().unwrap();
    instruction_data.append(BorshSerialize::try_to_vec(&proxy).unwrap().as_mut());
    data.insert(0, instruction_data);

    Instruction {
        program_id: space_wrapper::ID,
        accounts,
        data: data.into_iter().flatten().collect(),
    }
}

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

    #[allow(clippy::too_many_arguments)]
    async fn command_proxy_create_metadata_accounts(
        &self,
        rpc_client: &RpcClient,
        metadata_pubkey: Pubkey,
        mint: Pubkey,
        mint_authority: Pubkey,
        _payer: Pubkey,
        proxy_authority: Pubkey,
        authority: Pubkey,
        name: String,
        symbol: String,
        uri: String,
        creators: Vec<Creator>,
        seller_fee_basis_points: u16,
        _is_mutable: bool,
        collection: Option<Pubkey>,
        _uses: Uses,
        _collection_details: Option<CollectionDetails>,
    ) -> crate::Result<(u64, Vec<Instruction>)> {
        let minimum_balance_for_rent_exemption = rpc_client
            .get_minimum_balance_for_rent_exemption(std::mem::size_of::<
                mpl_token_metadata::state::Metadata,
            >())
            .await?;

        let instructions = vec![create_proxy_create_metadata_instruction(
            &proxy_authority,
            &metadata_pubkey,
            &mint,
            &mint_authority,
            &authority,
            name,
            symbol,
            uri,
            creators,
            seller_fee_basis_points,
            collection,
        )];

        Ok((minimum_balance_for_rent_exemption, instructions))
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Input {
    NoProxy {
        #[serde(with = "value::keypair")]
        update_authority: Keypair,
    },
    Proxy {
        #[serde(with = "value::pubkey")]
        proxy_as_update_authority: Pubkey,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InputStruct {
    pub is_mutable: bool,
    #[serde(with = "value::pubkey")]
    pub mint_account: Pubkey,
    #[serde(with = "value::pubkey")]
    pub mint_authority: Pubkey,
    #[serde(with = "value::keypair")]
    pub fee_payer: Keypair,
    pub metadata: NftMetadata,
    pub metadata_uri: String,
    pub uses: NftUses,
    // #[serde(with = "value::pubkey::opt")]
    // pub collection_mint_account: Option<Pubkey>,
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
const PROXY_AS_UPDATE_AUTHORITY: &str = "proxy_as_update_authority";
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
                required: false,
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
            CmdInput {
                name: PROXY_AS_UPDATE_AUTHORITY.into(),
                type_bounds: [ValueType::Pubkey].to_vec(),
                required: false,
                passthrough: true,
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

    async fn run(&self, ctx: Context, mut inputs: ValueSet) -> Result<ValueSet, CommandError> {
        match value::from_map(inputs.clone())? {
            Input::Proxy {
                proxy_as_update_authority: _,
            } => {
                let InputStruct {
                    is_mutable,
                    mint_account,
                    mint_authority,
                    fee_payer,
                    metadata,
                    metadata_uri,
                    uses,
                    // collection_mint_account,
                    creators,
                    collection_details,
                    submit,
                } = value::from_map(inputs.clone())?;

                // let collection = match collection_mint_account {
                //     Some(collection) => Some(Collection {
                //         verified: false,
                //         key:collection,
                //     }),
                //     _ => None,
                // };

                let collection = match inputs.remove("collection_mint_account") {
                    Some(Value::B32(collection)) => Some(Collection {
                        verified: false,
                        key: Pubkey::new_from_array(collection),
                    }),
                    _ => None,
                };

                let collection_details =
                    collection_details.map(|size| CollectionDetails::V1 { size });

                let (metadata_account, _) =
                    mpl_token_metadata::pda::find_metadata_account(&mint_account);

                let creators: Vec<Creator> = creators.into_iter().map(Creator::from).collect();

                let proxy_authority = find_proxy_authority_address(&fee_payer.pubkey());
                let (minimum_balance_for_rent_exemption, instructions) = self
                    .command_proxy_create_metadata_accounts(
                        &ctx.solana_client,
                        metadata_account,
                        mint_account,
                        mint_authority,
                        fee_payer.pubkey(),
                        proxy_authority,
                        fee_payer.pubkey(),
                        metadata.name,
                        metadata.symbol,
                        metadata_uri,
                        creators,
                        metadata.seller_fee_basis_points,
                        is_mutable,
                        collection.map(|c| c.key),
                        uses.into(),
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

                try_sign_wallet(&ctx, &mut transaction, &[&fee_payer], recent_blockhash).await?;

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
            Input::NoProxy { update_authority } => {
                let InputStruct {
                    is_mutable,
                    mint_account,
                    mint_authority,
                    fee_payer,
                    metadata,
                    metadata_uri,
                    uses,
                    creators,
                    collection_details,
                    submit,
                } = value::from_map(inputs.clone())?;

                let collection = match inputs.remove("collection_mint_account") {
                    Some(Value::B32(collection)) => Some(Collection {
                        verified: false,
                        key: Pubkey::new_from_array(collection),
                    }),
                    _ => None,
                };

                let collection_details =
                    collection_details.map(|size| CollectionDetails::V1 { size });

                let (metadata_account, _) =
                    mpl_token_metadata::pda::find_metadata_account(&mint_account);

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
    }
}

inventory::submit!(CommandDescription::new(CREATE_METADATA_ACCOUNT, |_| {
    Box::new(CreateMetadataAccount)
}));
