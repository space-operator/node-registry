use std::str::FromStr;

use crate::prelude::*;
use anchor_lang::Discriminator;
use borsh::BorshSerialize;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    system_program,
};

use space_wrapper::instruction::ProxyCreateMasterEditionV3 as Proxy;

#[derive(Debug, Clone)]
pub struct CreateMasterEdition;

pub fn create_proxy_create_master_edition_instruction(
    authority: &Pubkey,
    proxy_authority: &Pubkey,
    edition: &Pubkey,
    mint: &Pubkey,
    mint_authority: &Pubkey,
    metadata: &Pubkey,
    token_metadata_program: &Pubkey,
    token_program: &Pubkey,
    max_supply: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*authority, true),
        AccountMeta::new(*proxy_authority, false),
        AccountMeta::new(*edition, false),
        AccountMeta::new(*mint, false),
        AccountMeta::new_readonly(*mint_authority, true),
        AccountMeta::new(*metadata, false),
        AccountMeta::new_readonly(*token_metadata_program, false),
        AccountMeta::new_readonly(*token_program, false),
        AccountMeta::new_readonly(system_program::id(), false),
    ];

    let mut data = vec![max_supply.to_le_bytes().to_vec()];

    let proxy = Proxy {
        max_supply: Some(max_supply),
    };

    let mut instruction_data: Vec<u8> = Proxy::discriminator().try_to_vec().unwrap();
    instruction_data.append(BorshSerialize::try_to_vec(&proxy).unwrap().as_mut());
    data.insert(0, instruction_data);

    dbg!(Instruction {
        program_id: Pubkey::from_str("295QjveZJsZ198fYk9FTKaJLsgAWNdXKHsM6Qkb3dsVn").unwrap(),
        accounts,
        data: data.into_iter().flatten().collect(),
    })
}

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

    #[allow(clippy::too_many_arguments)]
    async fn command_proxy_create_metadata_accounts(
        &self,
        rpc_client: &RpcClient,
        metadata_pubkey: Pubkey,
        master_edition_pubkey: Pubkey,
        mint: Pubkey,
        mint_authority: Pubkey,
        payer: Pubkey,
        proxy_authority: Pubkey,
        max_supply: u64,
    ) -> crate::Result<(u64, Vec<Instruction>)> {
        let minimum_balance_for_rent_exemption = rpc_client
            .get_minimum_balance_for_rent_exemption(std::mem::size_of::<
                mpl_token_metadata::state::MasterEditionV2,
            >())
            .await?;

        let instructions = vec![create_proxy_create_master_edition_instruction(
            &payer,
            &proxy_authority,
            &master_edition_pubkey,
            &mint,
            &mint_authority,
            &metadata_pubkey,
            &mpl_token_metadata::id(),
            &spl_token::id(),
            max_supply,
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
    pub is_proxy: bool,
    #[serde(with = "value::pubkey::opt")]
    pub proxy_authority: Option<Pubkey>,
    #[serde(with = "value::keypair")]
    pub fee_payer: Keypair,
    #[serde(with = "value::keypair::opt")]
    pub update_authority: Option<Keypair>,
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
const IS_PROXY: &str = "is_proxy";
const PROXY_AUTHORITY: &str = "proxy_authority";
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

fn find_proxy_authority_address(authority: &Pubkey) -> Pubkey {
    let (expected_pda, bump_seed) = Pubkey::find_program_address(
        &[b"proxy", &authority.to_bytes()],
        &Pubkey::from_str("295QjveZJsZ198fYk9FTKaJLsgAWNdXKHsM6Qkb3dsVn").unwrap(),
    );

    let actual_pda = Pubkey::create_program_address(
        &[b"proxy", &authority.to_bytes(), &[bump_seed]],
        &Pubkey::from_str("295QjveZJsZ198fYk9FTKaJLsgAWNdXKHsM6Qkb3dsVn").unwrap(),
    )
    .unwrap();

    assert_eq!(expected_pda, actual_pda);
    dbg!(actual_pda)
}

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
                required: false,
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
            CmdInput {
                name: IS_PROXY.into(),
                type_bounds: [ValueType::Bool].to_vec(),
                required: true,
                passthrough: false,
            },
            CmdInput {
                name: PROXY_AUTHORITY.into(),
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
            CmdOutput {
                name: MASTER_EDITION_ACCOUNT.into(),
                r#type: ValueType::Pubkey,
            },
        ]
        .to_vec()
    }

    async fn run(&self, ctx: Context, inputs: ValueSet) -> Result<ValueSet, CommandError> {
        let Input {
            is_proxy,
            proxy_authority,
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

        if is_proxy {
            dbg!(proxy_authority);
            let proxy_authority = find_proxy_authority_address(&fee_payer.pubkey());
            dbg!(proxy_authority);
            let (minimum_balance_for_rent_exemption, instructions) = self
                .command_proxy_create_metadata_accounts(
                    &ctx.solana_client,
                    metadata_account,
                    master_edition_account,
                    mint_account,
                    mint_authority,
                    fee_payer.pubkey(),
                    proxy_authority,
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

            try_sign_wallet(&ctx, &mut transaction, &[&fee_payer], recent_blockhash).await?;

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
        } else {
            let (minimum_balance_for_rent_exemption, instructions) = self
                .command_create_master_edition(
                    &ctx.solana_client,
                    metadata_account,
                    master_edition_account,
                    mint_account,
                    mint_authority,
                    fee_payer.pubkey(),
                    update_authority.as_ref().unwrap().pubkey(),
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
                &[&update_authority.unwrap(), &fee_payer],
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
}

inventory::submit!(CommandDescription::new(
    CREATE_MASTER_EDITION,
    |_| Box::new(CreateMasterEdition)
));
