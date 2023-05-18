use std::mem::size_of;

use crate::prelude::*;
use anchor26::{InstructionData, ToAccountMetas};
use mpl_bubblegum::CreateTree;
use solana_program::{instruction::Instruction, system_instruction, system_program};
use solana_sdk::pubkey::Pubkey;
use spl_account_compression::{
    self, state::CONCURRENT_MERKLE_TREE_HEADER_SIZE_V1, ConcurrentMerkleTree,
};

// Command Name
const CREATE_TREE: &str = "create_tree";

const DEFINITION: &str =
    include_str!("../../../../node-definitions/solana/compression/create_tree.json");

fn build() -> Result<Box<dyn CommandTrait>, CommandError> {
    use once_cell::sync::Lazy;
    static CACHE: Lazy<Result<CmdBuilder, BuilderError>> = Lazy::new(|| {
        CmdBuilder::new(DEFINITION)?
            .check_name(CREATE_TREE)?
            .simple_instruction_info("signature")
    });
    Ok(CACHE.clone()?.build(run))
}

inventory::submit!(CommandDescription::new(CREATE_TREE, |_| { build() }));

#[derive(Serialize, Deserialize, Debug)]
pub struct Input {
    #[serde(with = "value::keypair")]
    pub payer: Keypair,
    #[serde(with = "value::keypair")]
    pub creator: Keypair,
    #[serde(with = "value::pubkey")]
    pub tree: Pubkey,
    #[serde(with = "value::pubkey")]
    pub merkle_tree: Pubkey,
    #[serde(default = "value::default::bool_true")]
    submit: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    #[serde(with = "value::signature::opt")]
    signature: Option<Signature>,
}

async fn run(mut ctx: Context, input: Input) -> Result<Output, CommandError> {
    let bubble_gum_program_id = mpl_bubblegum::id();

    // Allocate tree's account

    /// Only the following permutations are valid:
    ///
    /// | max_depth | max_buffer_size       |
    /// | --------- | --------------------- |
    /// | 14        | (64, 256, 1024, 2048) |           
    /// | 20        | (64, 256, 1024, 2048) |           
    /// | 24        | (64, 256, 512, 1024, 2048) |           
    /// | 26        | (64, 256, 512, 1024, 2048) |           
    /// | 30        | (512, 1024, 2048) |     
    const MAX_DEPTH: usize = 14;
    const MAX_BUFFER_SIZE: usize = 64;

    // To initialize a canopy on a ConcurrentMerkleTree account, you must initialize
    // the ConcurrentMerkleTree account with additional bytes. The number of additional bytes
    // needed is `(pow(2, N+1)-1) * 32`, where `N` is the number of levels of the merkle tree
    // you want the canopy to cache.
    //
    //https://github.com/solana-labs/solana-program-library/blob/9610bed5349f7a198903140cf2b74a727477b818/account-compression/programs/account-compression/src/canopy.rs
    //https://github.com/solana-labs/solana-program-library/blob/9610bed5349f7a198903140cf2b74a727477b818/account-compression/sdk/src/accounts/ConcurrentMerkleTreeAccount.ts#L209
    const merkle_tree_account_size: usize = CONCURRENT_MERKLE_TREE_HEADER_SIZE_V1
        + size_of::<ConcurrentMerkleTree<MAX_DEPTH, MAX_BUFFER_SIZE>>();

    let lamports = ctx
        .solana_client
        .get_minimum_balance_for_rent_exemption(account_size)
        .await?;

    let ix = system_instruction::create_account(
        &input.payer.pubkey(),
        &input.tree,
        lamports,
        u64::try_from(merkle_tree_account_size).unwrap(),
        &spl_account_compression::id(),
    );

    // Create Tree

    let seeds = &[input.tree.as_ref()];
    let tree_authority = Pubkey::find_program_address(seeds, &bubble_gum_program_id).0;

    let accounts = mpl_bubblegum::accounts::CreateTree {
        payer: input.payer.pubkey(),
        tree_authority,
        merkle_tree: input.merkle_tree,
        tree_creator: input.creator.pubkey(),
        log_wrapper: spl_noop::id(),
        system_program: system_program::id(),
        compression_program: spl_account_compression::id(),
    }
    .to_account_metas(None);

    let data = mpl_bubblegum::instruction::CreateTree {
        max_depth: todo!(),
        max_buffer_size: todo!(),
        public: todo!(),
    }
    .data();

    let minimum_balance_for_rent_exemption = ctx
        .solana_client
        .get_minimum_balance_for_rent_exemption(std::mem::size_of::<
            mpl_bubblegum::accounts::CreateTree,
        >())
        .await?;

    let ins = Instructions {
        fee_payer: input.payer.pubkey(),
        signers: [input.payer.clone_keypair(), input.creator.clone_keypair()].into(),
        instructions: [Instruction {
            program_id: mpl_bubblegum::id(),
            accounts,
            data,
        }]
        .into(),
        minimum_balance_for_rent_exemption,
    };

    let ins = input.submit.then_some(ins).unwrap_or_default();

    let signature = ctx
        .execute(
            ins,
            value::map! {
                "tree_authority" => tree_authority,
            },
        )
        .await?
        .signature;

    Ok(Output { signature })
}
