use crate::{nft::CandyMachineDataAlias, prelude::*};
use anchor_lang::{InstructionData, ToAccountMetas};
use solana_program::{
    instruction::Instruction, lamports, system_instruction, system_program, sysvar::rent,
};
use solana_sdk::pubkey::Pubkey;

use mpl_candy_machine_core::{candy_machine_core, instruction::Initialize, CandyMachineData};

// Command Name
const INITIALIZE_CANDY_MACHINE: &str = "initialize_candy_machine";

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
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    #[serde(with = "value::signature")]
    signature: Signature,
}

#[async_trait]
impl CommandTrait for InitializeCandyMachine {
    fn name(&self) -> Name {
        INITIALIZE_CANDY_MACHINE.into()
    }

    fn inputs(&self) -> Vec<CmdInput> {
        [
            CmdInput {
                name: "CANDY_MACHINE".into(),
                type_bounds: [ValueType::Pubkey, ValueType::Keypair].to_vec(),
                required: true,
                passthrough: true,
            },
            CmdInput {
                name: "AUTHORITY".into(),
                type_bounds: [ValueType::Pubkey, ValueType::Keypair, ValueType::String].to_vec(),
                required: true,
                passthrough: false,
            },
            CmdInput {
                name: "PAYER".into(),
                type_bounds: [ValueType::Keypair, ValueType::String].to_vec(),
                required: true,
                passthrough: true,
            },
            CmdInput {
                name: "COLLECTION_MINT".into(),
                type_bounds: [ValueType::Pubkey, ValueType::Keypair, ValueType::String].to_vec(),
                required: true,
                passthrough: false,
            },
            CmdInput {
                name: "COLLECTION_UPDATE_AUTHORITY".into(),
                type_bounds: [ValueType::Keypair, ValueType::String].to_vec(),
                required: true,
                passthrough: true,
            },
            CmdInput {
                name: "CANDY_MACHINE_DATA".into(),
                type_bounds: [ValueType::Free].to_vec(),
                required: true,
                passthrough: true,
            },
        ]
        .to_vec()
    }

    fn outputs(&self) -> Vec<CmdOutput> {
        [CmdOutput {
            name: "SIGNATURE".into(),
            r#type: ValueType::String,
        }]
        .to_vec()
    }

    async fn run(&self, ctx: Context, inputs: ValueSet) -> Result<ValueSet, CommandError> {
        let Input {
            candy_machine,
            authority,
            payer,
            collection_mint,
            collection_update_authority,
            candy_machine_data,
        } = value::from_map(inputs.clone())?;

        let token_metadata_program = mpl_token_metadata::id();
        let candy_machine_program = mpl_candy_machine_core::id();
        let candy_pubkey = candy_machine.pubkey();

        // Authority PDA
        let seeds = &["candy_machine".as_ref(), candy_pubkey.as_ref()];
        let authority_pda = Pubkey::find_program_address(seeds, &candy_machine_program).0;

        // Collection Metadata PDA
        let collection_metadata =
            mpl_token_metadata::pda::find_metadata_account(&collection_mint).0;

        // Master Edition PDA
        let collection_master_edition =
            mpl_token_metadata::pda::find_master_edition_account(&collection_mint).0;

        // Collection Authority PDA
        let collection_authority_record =
            mpl_token_metadata::pda::find_collection_authority_account(
                &collection_mint,
                &authority, // or authority_pda??
            )
            .0;

        let candy_machine_data = CandyMachineData::from(candy_machine_data);

        let accounts = mpl_candy_machine_core::accounts::Initialize {
            candy_machine: candy_pubkey,
            authority_pda,
            authority,
            payer: payer.pubkey(),
            collection_metadata,
            collection_mint,
            collection_master_edition,
            collection_update_authority: collection_update_authority.pubkey(),
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
            &payer.pubkey(),
            &candy_machine.pubkey(),
            lamports,
            candy_account_size as u64,
            &mpl_candy_machine_core::id(),
        );

        let instructions = vec![
            create_ix,
            Instruction {
                program_id: mpl_candy_machine_core::id(),
                accounts,
                data,
            },
        ];

        let minimum_balance_for_rent_exemption = ctx
            .solana_client
            .get_minimum_balance_for_rent_exemption(std::mem::size_of::<
                mpl_candy_machine_core::accounts::Initialize,
            >())
            .await?;

        let (mut transaction, recent_blockhash) = execute(
            &ctx.solana_client,
            &payer.pubkey(),
            &instructions,
            minimum_balance_for_rent_exemption,
        )
        .await?;

        try_sign_wallet(
            &ctx,
            &mut transaction,
            &[&payer, &candy_machine, &collection_update_authority],
            recent_blockhash,
        )
        .await?;

        let signature = submit_transaction(&ctx.solana_client, transaction).await?;

        Ok(value::to_map(&Output { signature })?)
    }
}

inventory::submit!(CommandDescription::new(INITIALIZE_CANDY_MACHINE, |_| {
    Box::new(InitializeCandyMachine)
}));

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
