use super::{NftCreator, NftMetadata, NftUses};
use crate::{prelude::*, proxy_authority::utils::find_proxy_authority_address};
use anchor_lang::InstructionData;
use mpl_token_metadata::state::{Collection, CollectionDetails, Creator};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    system_program,
};
use solana_sdk::pubkey::Pubkey;
use space_wrapper::instruction::ProxyCreateMetadataV3;

#[derive(Debug, Clone)]
pub struct CreateMetadataAccount;

impl CreateMetadataAccount {
    #[allow(clippy::too_many_arguments)]
    async fn create_metadata_accounts(
        &self,
        rpc_client: &RpcClient,
        inputs: &Input,
        metadata_account: Pubkey,
        update_authority: Pubkey,
        update_authority_is_signer: bool,
    ) -> crate::Result<(u64, Vec<Instruction>)> {
        let minimum_balance_for_rent_exemption = rpc_client
            .get_minimum_balance_for_rent_exemption(std::mem::size_of::<
                mpl_token_metadata::state::Metadata,
            >())
            .await?;

        let instruction = mpl_token_metadata::instruction::create_metadata_accounts_v3(
            mpl_token_metadata::ID,
            metadata_account,
            inputs.mint_account,
            inputs.mint_authority,
            inputs.fee_payer.pubkey(),
            update_authority,
            inputs.metadata.name.clone(),
            inputs.metadata.symbol.clone(),
            inputs.metadata_uri.clone(),
            Some(inputs.creators.iter().cloned().map(Creator::from).collect()),
            inputs.metadata.seller_fee_basis_points,
            update_authority_is_signer,
            inputs.is_mutable,
            inputs.collection_mint_account.map(|key| Collection {
                verified: false,
                key,
            }),
            Some(inputs.uses.clone().into()),
            inputs
                .collection_details
                .map(|size| CollectionDetails::V1 { size }),
        );

        Ok((minimum_balance_for_rent_exemption, [instruction].to_vec()))
    }

    #[allow(clippy::too_many_arguments)]
    async fn create_metadata_accounts_with_proxy(
        &self,
        rpc_client: &RpcClient,
        inputs: &Input,
        metadata_pubkey: Pubkey,
        proxy_authority: Pubkey,
    ) -> crate::Result<(u64, Vec<Instruction>)> {
        let minimum_balance_for_rent_exemption = rpc_client
            .get_minimum_balance_for_rent_exemption(std::mem::size_of::<
                mpl_token_metadata::state::Metadata,
            >())
            .await?;

        let instruction = Instruction {
            program_id: space_wrapper::ID,
            accounts: [
                AccountMeta::new_readonly(proxy_authority, false),
                AccountMeta::new(metadata_pubkey, false),
                AccountMeta::new_readonly(inputs.mint_account, false),
                AccountMeta::new(inputs.mint_authority, true),
                AccountMeta::new(inputs.fee_payer.pubkey(), true),
                AccountMeta::new_readonly(mpl_token_metadata::ID, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ]
            .to_vec(),
            data: ProxyCreateMetadataV3 {
                name: inputs.metadata.name.clone(),
                symbol: inputs.metadata.symbol.clone(),
                uri: inputs.metadata_uri.clone(),
                creators: inputs
                    .creators
                    .iter()
                    .cloned()
                    .map(|c| space_wrapper::state::creator::Creator {
                        key: c.address,
                        verified: c.verified.unwrap_or(false),
                        share: c.share,
                    })
                    .collect(),
                seller_fee_basis_points: inputs.metadata.seller_fee_basis_points,
                collection: inputs.collection_mint_account,
            }
            .data(),
        };

        Ok((minimum_balance_for_rent_exemption, [instruction].to_vec()))
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum UpdateAuthority {
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
pub struct Input {
    #[serde(flatten)]
    pub update_authority: UpdateAuthority,
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
    #[serde(default, with = "value::pubkey::opt")]
    pub collection_mint_account: Option<Pubkey>,
    pub creators: Vec<NftCreator>,
    pub collection_details: Option<u64>,
    #[serde(default = "value::default::bool_true")]
    pub submit: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    #[serde(default, with = "value::signature::opt")]
    signature: Option<Signature>,
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
    fn instruction_info(&self) -> Option<InstructionInfo> {
        Some(InstructionInfo::simple(self, SIGNATURE))
    }

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
                type_bounds: [ValueType::Pubkey].to_vec(),
                required: true,
                passthrough: false,
            },
            CmdInput {
                name: MINT_AUTHORITY.into(),
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
                name: UPDATE_AUTHORITY.into(),
                type_bounds: [ValueType::Keypair].to_vec(),
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
                type_bounds: [ValueType::Pubkey].to_vec(),
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

    async fn run(&self, mut ctx: Context, inputs: ValueSet) -> Result<ValueSet, CommandError> {
        let inputs: Input = value::from_map(inputs)?;

        let (metadata_account, _) =
            mpl_token_metadata::pda::find_metadata_account(&inputs.mint_account);

        let (minimum_balance_for_rent_exemption, instructions, signers) =
            match &inputs.update_authority {
                // TODO: this input is unused
                UpdateAuthority::Proxy { .. } => {
                    let proxy_authority = find_proxy_authority_address(&inputs.fee_payer.pubkey());
                    let (minimum_balance_for_rent_exemption, instructions) = self
                        .create_metadata_accounts_with_proxy(
                            &ctx.solana_client,
                            &inputs,
                            metadata_account,
                            proxy_authority,
                        )
                        .await?;
                    (
                        minimum_balance_for_rent_exemption,
                        instructions,
                        vec![inputs.fee_payer.clone_keypair()],
                    )
                }
                UpdateAuthority::NoProxy { update_authority } => {
                    let (minimum_balance_for_rent_exemption, instructions) = self
                        .create_metadata_accounts(
                            &ctx.solana_client,
                            &inputs,
                            metadata_account,
                            update_authority.pubkey(),
                            true,
                        )
                        .await?;
                    (
                        minimum_balance_for_rent_exemption,
                        instructions,
                        vec![
                            inputs.fee_payer.clone_keypair(),
                            update_authority.clone_keypair(),
                        ],
                    )
                }
            };

        let instructions = if inputs.submit {
            Instructions {
                fee_payer: inputs.fee_payer.pubkey(),
                signers,
                minimum_balance_for_rent_exemption,
                instructions,
            }
        } else {
            Instructions::default()
        };

        let signature = ctx
            .execute(
                instructions,
                value::map! {
                    METADATA_ACCOUNT => metadata_account,
                },
            )
            .await?
            .signature;

        Ok(value::to_map(&Output { signature })?)
    }
}

inventory::submit!(CommandDescription::new(CREATE_METADATA_ACCOUNT, |_| {
    Box::new(CreateMetadataAccount)
}));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inputs() {
        let metadata: Value = serde_json::from_str::<serde_json::Value>(
            r#"
{
    "name": "SO #11111",
    "symbol": "SPOP",
    "description": "Space Operator is a dynamic PFP collection",
    "seller_fee_basis_points": 250,
    "image": "https://arweave.net/vb1tD7tfAyrhZceA1MOYvvyqzZWgzHGDVZF37yDNH1Q",
    "attributes": [
        {
            "trait_type": "Season",
            "value": "Fall"
        },
        {
            "trait_type": "Light Color",
            "value": "Orange"
        }
    ],
    "properties": {
        "files": [
            {
                "uri": "https://arweave.net/vb1tD7tfAyrhZceA1MOYvvyqzZWgzHGDVZF37yDNH1Q",
                "type": "image/jpeg"
            }
        ],
        "category": null
    }
}"#,
        )
        .unwrap()
        .into();
        let uses: Value = serde_json::from_str::<serde_json::Value>(
            r#"
{
"use_method": "Burn",
"remaining": 500,
"total": 500
}
"#,
        )
        .unwrap()
        .into();
        let creators: Value = serde_json::from_str::<serde_json::Value>(
            r#"
[{
"address": "DpfvhHU7z1CK8eP5xbEz8c4WBNHUfqUVtAE7opP2kJBc",
"share": 100
}]"#,
        )
        .unwrap()
        .into();
        let inputs = value::map! {
            PROXY_AS_UPDATE_AUTHORITY => "3G3ixjPdvg7NhazP932tCk88jgLJLzaDBe84mPa43Zyp",
            IS_MUTABLE => true,
            MINT_ACCOUNT => "C3EbZLYQ7Axv4PS9o4s4bSruFaiAVcynHZYds18VyWdZ",
            MINT_AUTHORITY => "C3EbZLYQ7Axv4PS9o4s4bSruFaiAVcynHZYds18VyWdZ",
            FEE_PAYER => "5s8bKTTgKLh2TudJBQwU6sx9DfFEtHcBP85aYZquEsqHrvipcWWCXxuyz4fsGsxTZ8NGMqMHFowUoQcoqcJSwLrP",
            METADATA => metadata,
            METADATA_URI => "https://arweave.net/3FxpIIbpySnfTTXIrpojhF2KHHjevI8Mrt3pACmEbSY",
            USES => uses,
            CREATORS => creators,
        };
        let inputs: Input = value::from_map(inputs).unwrap();
        dbg!(inputs);
    }
}
