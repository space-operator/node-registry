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

// Command Name
const NAME: &str = "create_metadata_account";

const DEFINITION: &str =
    include_str!("../../../../node-definitions/solana/NFT/create_metadata_account.json");

fn build() -> Result<Box<dyn CommandTrait>, CommandError> {
    use once_cell::sync::Lazy;
    static CACHE: Lazy<Result<CmdBuilder, BuilderError>> = Lazy::new(|| {
        CmdBuilder::new(DEFINITION)?
            .check_name(NAME)?
            .simple_instruction_info("signature")
    });
    Ok(CACHE.clone()?.build(run))
}

inventory::submit!(CommandDescription::new(NAME, |_| { build() }));

async fn create_metadata_accounts(
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
        inputs.uses.clone().map(Into::into),
        inputs
            .collection_details
            .map(|size| CollectionDetails::V1 { size }),
    );

    Ok((minimum_balance_for_rent_exemption, [instruction].to_vec()))
}

async fn create_metadata_accounts_with_proxy(
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
    pub uses: Option<NftUses>,
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

async fn run(mut ctx: Context, input: Input) -> Result<Output, CommandError> {
    let (metadata_account, _) = mpl_token_metadata::pda::find_metadata_account(&input.mint_account);

    let (minimum_balance_for_rent_exemption, instructions, signers) = match &input.update_authority
    {
        // TODO: this input is unused
        UpdateAuthority::Proxy { .. } => {
            let proxy_authority = find_proxy_authority_address(&input.fee_payer.pubkey());
            let (minimum_balance_for_rent_exemption, instructions) =
                create_metadata_accounts_with_proxy(
                    &ctx.solana_client,
                    &input,
                    metadata_account,
                    proxy_authority,
                )
                .await?;
            (
                minimum_balance_for_rent_exemption,
                instructions,
                vec![input.fee_payer.clone_keypair()],
            )
        }
        UpdateAuthority::NoProxy { update_authority } => {
            let (minimum_balance_for_rent_exemption, instructions) = create_metadata_accounts(
                &ctx.solana_client,
                &input,
                metadata_account,
                update_authority.pubkey(),
                true,
            )
            .await?;
            (
                minimum_balance_for_rent_exemption,
                instructions,
                vec![
                    input.fee_payer.clone_keypair(),
                    update_authority.clone_keypair(),
                ],
            )
        }
    };

    let ins = Instructions {
        fee_payer: input.fee_payer.pubkey(),
        signers,
        minimum_balance_for_rent_exemption,
        instructions,
    };

    let ins = input.submit.then_some(ins).unwrap_or_default();

    let signature = ctx
        .execute(
            ins,
            value::map! {
                "metadata_account" => metadata_account,
            },
        )
        .await?
        .signature;

    Ok(Output { signature })
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_inputs() {
//         let metadata: Value = serde_json::from_str::<serde_json::Value>(
//             r#"
// {
//     "name": "SO #11111",
//     "symbol": "SPOP",
//     "description": "Space Operator is a dynamic PFP collection",
//     "seller_fee_basis_points": 250,
//     "image": "https://arweave.net/vb1tD7tfAyrhZceA1MOYvvyqzZWgzHGDVZF37yDNH1Q",
//     "attributes": [
//         {
//             "trait_type": "Season",
//             "value": "Fall"
//         },
//         {
//             "trait_type": "Light Color",
//             "value": "Orange"
//         }
//     ],
//     "properties": {
//         "files": [
//             {
//                 "uri": "https://arweave.net/vb1tD7tfAyrhZceA1MOYvvyqzZWgzHGDVZF37yDNH1Q",
//                 "type": "image/jpeg"
//             }
//         ],
//         "category": null
//     }
// }"#,
//         )
//         .unwrap()
//         .into();
//         let uses: Value = serde_json::from_str::<serde_json::Value>(
//             r#"
// {
// "use_method": "Burn",
// "remaining": 500,
// "total": 500
// }
// "#,
//         )
//         .unwrap()
//         .into();
//         let creators: Value = serde_json::from_str::<serde_json::Value>(
//             r#"
// [{
// "address": "DpfvhHU7z1CK8eP5xbEz8c4WBNHUfqUVtAE7opP2kJBc",
// "share": 100
// }]"#,
//         )
//         .unwrap()
//         .into();
//         let inputs = value::map! {
//             PROXY_AS_UPDATE_AUTHORITY => "3G3ixjPdvg7NhazP932tCk88jgLJLzaDBe84mPa43Zyp",
//             IS_MUTABLE => true,
//             MINT_ACCOUNT => "C3EbZLYQ7Axv4PS9o4s4bSruFaiAVcynHZYds18VyWdZ",
//             MINT_AUTHORITY => "C3EbZLYQ7Axv4PS9o4s4bSruFaiAVcynHZYds18VyWdZ",
//             FEE_PAYER => "5s8bKTTgKLh2TudJBQwU6sx9DfFEtHcBP85aYZquEsqHrvipcWWCXxuyz4fsGsxTZ8NGMqMHFowUoQcoqcJSwLrP",
//             METADATA => metadata,
//             METADATA_URI => "https://arweave.net/3FxpIIbpySnfTTXIrpojhF2KHHjevI8Mrt3pACmEbSY",
//             USES => uses,
//             CREATORS => creators,
//         };
//         let inputs: Input = value::from_map(inputs).unwrap();
//         dbg!(inputs);
//     }
// }
