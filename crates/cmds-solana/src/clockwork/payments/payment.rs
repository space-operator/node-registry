use std::str::FromStr;

use crate::prelude::*;
use borsh::BorshSerialize;
use mpl_token_metadata::state::{Collection, CollectionDetails, Creator, Uses};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    system_program,
};
use solana_sdk::pubkey::Pubkey;
use space_wrapper::instruction::ProxyCreateMetadataV3 as Proxy;

#[derive(Debug)]
pub struct Payment;

// pub fn create_proxy_create_metadata_instruction(
//     proxy_authority: &Pubkey,
//     metadata: &Pubkey,
//     mint: &Pubkey,
//     mint_authority: &Pubkey,
//     authority: &Pubkey,
//     name: String,
//     symbol: String,
//     uri: String,
//     creators: Vec<Creator>,
//     seller_fee_basis_points: u16,
//     collection: Option<Pubkey>,
// ) -> Instruction {
//     let accounts = vec![
//         AccountMeta::new_readonly(*proxy_authority, false),
//         AccountMeta::new(*metadata, false),
//         AccountMeta::new_readonly(*mint, false),
//         AccountMeta::new(*mint_authority, true),
//         AccountMeta::new(*authority, true),
//         AccountMeta::new_readonly(mpl_token_metadata::id(), false),
//         AccountMeta::new_readonly(system_program::ID, false),
//     ];

//     let mut data = vec![
//         name.as_bytes().to_vec(),
//         symbol.as_bytes().to_vec(),
//         uri.as_bytes().to_vec(),
//     ];

//     for creator in creators.clone() {
//         let serialized = creator.try_to_vec().expect("should serialize");
//         let serialized_slice = serialized.to_owned();
//         data.push(serialized_slice);
//     }

//     let seller_fee_basis_points_bytes = seller_fee_basis_points.to_le_bytes().to_vec();
//     data.push(seller_fee_basis_points_bytes);

//     let collection_bytes = collection.try_to_vec().expect("should serialize");
//     let collection_bytes = collection_bytes.to_vec();
//     data.push(collection_bytes);

//     // Create method signature hash
//     let proxy = Proxy {
//         name,
//         symbol,
//         uri,
//         creators: vec![space_wrapper::state::creator::Creator {
//             key: creators[0].address,
//             verified: creators[0].verified,
//             share: creators[0].share,
//         }],
//         seller_fee_basis_points,
//         collection,
//     };

//     let mut instruction_data: Vec<u8> = Proxy::discriminator().try_to_vec().unwrap();
//     instruction_data.append(BorshSerialize::try_to_vec(&proxy).unwrap().as_mut());
//     data.insert(0, instruction_data);

//     Instruction {
//         program_id: Pubkey::from_str("295QjveZJsZ198fYk9FTKaJLsgAWNdXKHsM6Qkb3dsVn").unwrap(),
//         accounts,
//         data: data.into_iter().flatten().collect(),
//     }
// }

fn create_payment(
    authority_token_account: Pubkey,
    mint: Pubkey,
    payment: Pubkey,
    thread: Pubkey,
    recipient: Pubkey,
    recipient_ata_pubkey: Pubkey,
) -> Instruction {
    // create ix
    let create_payment_ix = Instruction {
        program_id: payments::ID,
        accounts: vec![
            AccountMeta::new_readonly(associated_token::ID, false),
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new(authority_token_account, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new(payment, false),
            AccountMeta::new_readonly(recipient, false),
            AccountMeta::new_readonly(sysvar::rent::ID, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(token::ID, false),
        ],
        data: payments::instruction::CreatePayment { amount: 10_000 }.data(),
    }
}

impl Payment {
    #[allow(clippy::too_many_arguments)]
    async fn command_create_payment(
        &self,
        rpc_client: &RpcClient,
        metadata_pubkey: Pubkey,
        mint: Pubkey,
        mint_authority: Pubkey,
        payer: Pubkey,
        proxy_authority: Pubkey,
        authority: Pubkey,
        name: String,
        symbol: String,
        uri: String,
        creators: Vec<Creator>,
        seller_fee_basis_points: u16,
        is_mutable: bool,
        collection: Option<Pubkey>,
        uses: Uses,
        collection_details: Option<CollectionDetails>,
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
    IsImmediate {
        is_immediate: bool,
    },
    Schedule {
        #[serde(with = "value::pubkey")]
        proxy_as_update_authority: Pubkey,
    },
    MonitorAccount {
        #[serde(with = "value::pubkey")]
        monitor_account: Pubkey,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InputStruct {
    #[serde(with = "value::keypair")]
    pub payer: Keypair,
    #[serde(with = "value::pubkey")]
    pub token_mint: Pubkey,
    #[serde(with = "value::pubkey")]
    pub recipient: Pubkey,
    pub amount: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    #[serde(with = "value::signature")]
    signature: Signature,
    #[serde(with = "value::pubkey")]
    metadata_account: Pubkey,
}

// Command Name
const CREATE_PAYMENT: &str = "create_payment";

// Inputs

const IS_IMMEDIATE: &str = "is_immediate";
const SCHEDULE: &str = "schedule";
const IS_SKIPPABLE: &str = "is_skippable";
const MONITOR_ACCOUNT: &str = "monitor_account";
const PAYER: &str = "payer";
const TOKEN_MINT: &str = "token_mint";
const RECIPIENT: &str = "recipient";
const AMOUNT: &str = "amount";

// Outputs
const SIGNATURE: &str = "signature";
const THREAD: &str = "thread";

// TODO
// convert schedule
// /home/amir/.cargo/registry/src/github.com-1ecc6299db9ec823/clockwork-cron-1.4.0/src/schedule.rs

#[async_trait]
impl CommandTrait for Payment {
    fn name(&self) -> Name {
        CREATE_PAYMENT.into()
    }

    fn inputs(&self) -> Vec<CmdInput> {
        [
            CmdInput {
                name: IS_IMMEDIATE.into(),
                type_bounds: [ValueType::Bool].to_vec(),
                required: false,
                passthrough: false,
            },
            CmdInput {
                name: SCHEDULE.into(),
                type_bounds: [ValueType::String].to_vec(),
                required: false,
                passthrough: false,
            },
            CmdInput {
                name: IS_SKIPPABLE.into(),
                type_bounds: [ValueType::Bool].to_vec(),
                required: false,
                passthrough: false,
            },
            CmdInput {
                name: MONITOR_ACCOUNT.into(),
                type_bounds: [ValueType::Keypair, ValueType::String, ValueType::Pubkey].to_vec(),
                required: false,
                passthrough: false,
            },
            CmdInput {
                name: PAYER.into(),
                type_bounds: [ValueType::Keypair, ValueType::String].to_vec(),
                required: true,
                passthrough: true,
            },
            CmdInput {
                name: TOKEN_MINT.into(),
                type_bounds: [ValueType::Pubkey, ValueType::Keypair, ValueType::String].to_vec(),
                required: true,
                passthrough: false,
            },
            CmdInput {
                name: RECIPIENT.into(),
                type_bounds: [ValueType::Keypair, ValueType::String, ValueType::Pubkey].to_vec(),
                required: true,
                passthrough: false,
            },
            CmdInput {
                name: AMOUNT.into(),
                type_bounds: [ValueType::U64].to_vec(),
                required: true,
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
                name: THREAD.into(),
                r#type: ValueType::Pubkey,
            },
        ]
        .to_vec()
    }

    async fn run(&self, ctx: Context, inputs: ValueSet) -> Result<ValueSet, CommandError> {
        match value::from_map(inputs.clone())? {
            Input::IsImmediate { is_immediate } => {
                let InputStruct {
                    payer,
                    token_mint,
                    recipient,
                    amount,
                } = value::from_map(inputs)?;

                let (minimum_balance_for_rent_exemption, instructions) = self
                    .command_create_payment(
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

                let signature = submit_transaction(&ctx.solana_client, transaction).await?;

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
                    collection_mint_account,
                    creators,
                    collection_details,
                    submit,
                } = value::from_map(inputs)?;

                let collection_details =
                    collection_details.map(|size| CollectionDetails::V1 { size });

                let (metadata_account, _) =
                    mpl_token_metadata::pda::find_metadata_account(&mint_account);

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
    }
}

inventory::submit!(CommandDescription::new(CREATE_PAYMENT, |_| {
    Box::new(Payment)
}));
