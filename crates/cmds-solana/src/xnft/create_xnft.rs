use std::str::FromStr;

use crate::prelude::*;
use anchor_lang::ToAccountMetas;
use solana_program::{instruction::Instruction, system_program};
use solana_sdk::pubkey::Pubkey;
use spl_associated_token_account::get_associated_token_address;

// Command Name
const CREATE_XNFT: &str = "create_xnft";

const DEFINITION: &str = include_str!("../../../../node-definitions/solana/xnft/create_xnft.json");

fn build() -> Result<Box<dyn CommandTrait>, CommandError> {
    use once_cell::sync::Lazy;
    static CACHE: Lazy<Result<CmdBuilder, BuilderError>> = Lazy::new(|| {
        CmdBuilder::new(DEFINITION)?
            .check_name(CREATE_XNFT)?
            .simple_instruction_info("signature")
    });
    Ok(CACHE.clone()?.build(run))
}

inventory::submit!(CommandDescription::new(CREATE_XNFT, |_| { build() }));

#[derive(Serialize, Deserialize, Debug)]
pub struct Input {
    #[serde(with = "value::keypair")]
    pub payer: Keypair,
    #[serde(with = "value::keypair")]
    pub authority: Keypair,
    #[serde(default = "value::default::bool_true")]
    submit: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    #[serde(with = "value::signature::opt")]
    signature: Option<Signature>,
    //TODO
}

async fn run(mut ctx: Context, input: Input) -> Result<Output, CommandError> {
    let xnft_program_id = xnft::id();
    //https://github.com/coral-xyz/xnft/blob/master/tests/common.ts
    let metadata_program = Pubkey::from_str("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s").unwrap();

    // Master Mint PDA
    let authority = &input.authority.pubkey();
    let seeds = &["mint".as_ref(), authority.as_ref()];
    let master_mint = Pubkey::find_program_address(seeds, &xnft_program_id).0;

    // Master Token
    let master_token = get_associated_token_address(&authority, &master_mint);

    // xNFT PDA
    let seeds = &["xnft".as_ref(), master_mint.as_ref()];
    let master_mint = Pubkey::find_program_address(seeds, &xnft_program_id).0;

    // Master Metadata
    let master_metadata = Pubkey::create_program_address(
        &[
            "metadata".as_ref(),
            metadata_program.to_bytes().as_ref(),
            master_mint.as_ref(),
        ],
        &metadata_program,
    );

    let accounts = xnft::accounts::CreateAppXnft {
        master_mint: todo!(),
        master_token: todo!(),
        master_metadata: todo!(),
        xnft: todo!(),
        payer: todo!(),
        publisher: todo!(),
        system_program: todo!(),
        token_program: todo!(),
        associated_token_program: todo!(),
        metadata_program: todo!(),
        rent: todo!(),
    }
    .to_account_metas(None);

    let metadata = input.metadata.into();

    let data = mpl_bubblegum::instruction::MintV1 { message: metadata }.data();

    let minimum_balance_for_rent_exemption = ctx
        .solana_client
        .get_minimum_balance_for_rent_exemption(
            std::mem::size_of::<mpl_bubblegum::accounts::MintV1>(),
        )
        .await?;

    let ins = Instructions {
        fee_payer: input.payer.pubkey(),
        signers: [
            input.payer.clone_keypair(),
            input.tree_delegate.clone_keypair(),
        ]
        .into(),
        instructions: [Instruction {
            program_id: mpl_bubblegum::id(),
            accounts,
            data,
        }]
        .into(),
        minimum_balance_for_rent_exemption,
    };

    let ins = input.submit.then_some(ins).unwrap_or_default();

    let signature = ctx.execute(ins, <_>::default()).await?.signature;

    Ok(Output { signature })
}
