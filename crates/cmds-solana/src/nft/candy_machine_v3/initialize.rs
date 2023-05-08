use crate::{nft::CandyMachineDataAlias, prelude::*};
use anchor_lang::{InstructionData, ToAccountMetas};
use solana_program::{instruction::Instruction, system_instruction, system_program};
use solana_sdk::pubkey::Pubkey;

use mpl_candy_machine_core::{instruction::Initialize, CandyMachineData};

// Command Name
const INITIALIZE_CANDY_MACHINE: &str = "initialize_candy_machine";

const DEFINITION: &str =
    include_str!("../../../../../node-definitions/solana/NFT/candy_machine/initialize.json");

fn build() -> Result<Box<dyn CommandTrait>, CommandError> {
    use once_cell::sync::Lazy;
    static CACHE: Lazy<Result<CmdBuilder, BuilderError>> = Lazy::new(|| {
        CmdBuilder::new(DEFINITION)?
            .check_name(INITIALIZE_CANDY_MACHINE)?
            .simple_instruction_info("signature")
    });
    Ok(CACHE.clone()?.build(run))
}

inventory::submit!(CommandDescription::new(INITIALIZE_CANDY_MACHINE, |_| {
    build()
}));

#[derive(Debug)]
pub struct InitializeCandyMachine;

#[derive(Serialize, Deserialize, Debug)]
pub struct Input {
    #[serde(with = "value::keypair")]
    pub candy_machine: Keypair,
    #[serde(with = "value::pubkey")]
    pub authority: Pubkey,
    #[serde(with = "value::keypair")]
    pub payer: Keypair,
    #[serde(with = "value::pubkey")]
    pub collection_mint: Pubkey,
    #[serde(with = "value::keypair")]
    pub collection_update_authority: Keypair,
    pub candy_machine_data: CandyMachineDataAlias,
    #[serde(default = "value::default::bool_true")]
    submit: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    #[serde(default, with = "value::signature::opt")]
    signature: Option<Signature>,
}

async fn run(mut ctx: Context, input: Input) -> Result<Output, CommandError> {
    let token_metadata_program = mpl_token_metadata::id();
    let candy_machine_program = mpl_candy_machine_core::id();
    let candy_pubkey = input.candy_machine.pubkey();

    // Authority PDA
    let seeds = &["candy_machine".as_ref(), candy_pubkey.as_ref()];
    let authority_pda = Pubkey::find_program_address(seeds, &candy_machine_program).0;

    // Collection Metadata PDA
    let collection_metadata =
        mpl_token_metadata::pda::find_metadata_account(&input.collection_mint).0;

    // Master Edition PDA
    let collection_master_edition =
        mpl_token_metadata::pda::find_master_edition_account(&input.collection_mint).0;

    // Collection Authority PDA
    let collection_authority_record = mpl_token_metadata::pda::find_collection_authority_account(
        &input.collection_mint,
        &input.authority, // or authority_pda??
    )
    .0;

    let candy_machine_data = CandyMachineData::from(input.candy_machine_data);

    let accounts = mpl_candy_machine_core::accounts::Initialize {
        candy_machine: candy_pubkey,
        authority_pda,
        authority: input.authority,
        payer: input.payer.pubkey(),
        collection_metadata,
        collection_mint: input.collection_mint,
        collection_master_edition,
        collection_update_authority: input.collection_update_authority.pubkey(),
        collection_authority_record,
        token_metadata_program,
        system_program: system_program::ID,
    }
    .to_account_metas(None);

    let data = Initialize {
        data: candy_machine_data.clone(),
    }
    .data();

    let candy_account_size = candy_machine_data.get_space_for_candy().unwrap_or(216);

    let lamports = ctx
        .solana_client
        .get_minimum_balance_for_rent_exemption(candy_account_size)
        .await?;

    let create_ix = system_instruction::create_account(
        &input.payer.pubkey(),
        &input.candy_machine.pubkey(),
        lamports,
        candy_account_size as u64,
        &mpl_candy_machine_core::id(),
    );

    let minimum_balance_for_rent_exemption = ctx
        .solana_client
        .get_minimum_balance_for_rent_exemption(std::mem::size_of::<
            mpl_candy_machine_core::accounts::Initialize,
        >())
        .await?;

    let ins = Instructions {
        fee_payer: input.payer.pubkey(),
        signers: [
            input.payer.clone_keypair(),
            input.candy_machine.clone_keypair(),
            input.collection_update_authority.clone_keypair(),
        ]
        .into(),
        instructions: [
            create_ix,
            Instruction {
                program_id: mpl_candy_machine_core::id(),
                accounts,
                data,
            },
        ]
        .into(),
        minimum_balance_for_rent_exemption,
    };

    let ins = input.submit.then_some(ins).unwrap_or_default();

    let signature = ctx.execute(ins, <_>::default()).await?.signature;

    Ok(Output { signature })
}

// {
//     "items_available": 10,
//     "symbol": "CORE",
//     "seller_fee_basis_points": 500,
//     "max_supply": 0,
//     "is_mutable": true,
//     "creators": [
//       {
//         "address": "2gdutJtCz1f9P3NJGP4HbBYFCHMh8rVAhmT2QDSb9dN9",
//         "verified": false,
//         "share": 100
//       }],
//     "config_line_settings": {
//       "prefix_name": "TEST",
//       "name_length": 10,
//       "prefix_uri": "https://arweave.net/",
//       "uri_length": 50,
//       "is_sequential": false
//     },
//     "hiddenSettings": null
//   }
