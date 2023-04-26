use crate::prelude::Pubkey;
use mpl_token_metadata::state::{Collection, Creator, DataV2, UseMethod, Uses};
use serde::{Deserialize, Serialize};

pub mod approve_collection_authority;
pub mod approve_use_authority;
pub mod arweave_file_upload;
pub mod arweave_nft_upload;
pub mod create_master_edition;
pub mod create_metadata_account;
pub mod get_left_uses;
pub mod sign_metadata;
pub mod update_metadata_account;
pub mod verify_collection;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NftDataV2 {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub seller_fee_basis_points: u16,
    pub creators: Option<Vec<NftCreator>>,
    pub collection: Option<NftCollection>,
    pub uses: Option<NftUses>,
}

impl From<NftDataV2> for DataV2 {
    fn from(v: NftDataV2) -> Self {
        Self {
            name: v.name,
            symbol: v.symbol,
            uri: v.uri,
            seller_fee_basis_points: v.seller_fee_basis_points,
            creators: v.creators.map(|v| v.into_iter().map(Into::into).collect()),
            collection: v.collection.map(Into::into),
            uses: v.uses.map(Into::into),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NftCollection {
    pub verified: bool,
    #[serde(with = "value::pubkey")]
    pub key: Pubkey,
}

impl From<NftCollection> for Collection {
    fn from(v: NftCollection) -> Self {
        Self {
            verified: v.verified,
            key: v.key,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NftMetadata {
    pub name: String,
    pub symbol: String,
    pub description: String,
    pub seller_fee_basis_points: u16,
    pub image: String,
    pub animation_url: Option<String>,
    pub external_url: Option<String>,
    pub attributes: Vec<NftMetadataAttribute>,
    pub properties: Option<NftMetadataProperties>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NftMetadataAttribute {
    pub trait_type: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NftMetadataProperties {
    pub files: Option<Vec<NftMetadataFile>>,
    pub category: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NftMetadataFile {
    pub uri: String,
    #[serde(rename = "type")]
    pub kind: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NftCreator {
    #[serde(with = "value::pubkey")]
    pub address: Pubkey,
    pub verified: Option<bool>,
    pub share: u8, // in percentage not basis points
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NftUses {
    pub use_method: NftUseMethod,
    pub remaining: u64,
    pub total: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum NftUseMethod {
    Burn,
    Single,
    Multiple,
}

impl From<NftUses> for Uses {
    fn from(v: NftUses) -> Self {
        Uses {
            use_method: UseMethod::from(v.use_method.clone()),
            remaining: v.remaining,
            total: v.total,
        }
    }
}

impl From<NftUseMethod> for UseMethod {
    fn from(v: NftUseMethod) -> Self {
        match v {
            NftUseMethod::Burn => UseMethod::Burn,
            NftUseMethod::Single => UseMethod::Single,
            NftUseMethod::Multiple => UseMethod::Multiple,
        }
    }
}

impl From<NftCreator> for Creator {
    fn from(v: NftCreator) -> Self {
        Creator {
            address: v.address,
            verified: v.verified.is_some(),
            share: v.share,
        }
    }
}
