use crate::wormhole::{ForeignAddress, PostVAAData, VAA};
use std::str::FromStr;

use crate::prelude::*;

use borsh::BorshSerialize;
use primitive_types::U256;
use rand::Rng;
use solana_program::{instruction::AccountMeta, system_program, sysvar};
use solana_sdk::pubkey::Pubkey;
use wormhole_sdk::{token::Message, Address};

use super::{NFTBridgeInstructions, TransferNativeData, TransferWrappedData};

// Command Name
const NAME: &str = "nft_transfer_native";

const DEFINITION: &str = include_str!(
    "../../../../../node-definitions/solana/wormhole/nft_bridge/nft_transfer_native.json"
);

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

#[derive(Serialize, Deserialize, Debug)]
pub struct Input {
    #[serde(with = "value::keypair")]
    pub payer: Keypair,
    pub target_address: Address,
    pub target_chain: u16,
    #[serde(with = "value::keypair")]
    pub message: Keypair,
    #[serde(with = "value::pubkey")]
    pub from: Pubkey,
    #[serde(with = "value::pubkey")]
    pub mint: Pubkey,

    #[serde(default = "value::default::bool_true")]
    submit: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    #[serde(with = "value::signature::opt")]
    signature: Option<Signature>,
}

async fn run(mut ctx: Context, input: Input) -> Result<Output, CommandError> {
    let wormhole_core_program_id = match ctx.cfg.solana_client.cluster {
        SolanaNet::Mainnet => Pubkey::from_str("worm2ZoG2kUd4vFXhvjh93UUH596ayRfgQ2MgjNMTth")?,
        // TODO testnet not deployed yet
        SolanaNet::Testnet => Pubkey::from_str("3u8hJUVTA4jH1wYAyUur7FFZVQ8H635K3tSHHF4ssjQ5")?,
        SolanaNet::Devnet => Pubkey::from_str("3u8hJUVTA4jH1wYAyUur7FFZVQ8H635K3tSHHF4ssjQ5")?,
    };

    let nft_bridge_program_id = match ctx.cfg.solana_client.cluster {
        SolanaNet::Mainnet => Pubkey::from_str("WnFt12ZrnzZrFZkt2xsNsaNWoQribnuQ5B5FrDbwDhD")?,
        // TODO testnet not deployed yet
        SolanaNet::Testnet => Pubkey::from_str("0x4a8bc80Ed5a4067f1CCf107057b8270E0cC11A78")?,
        SolanaNet::Devnet => Pubkey::from_str("0x4a8bc80Ed5a4067f1CCf107057b8270E0cC11A78")?,
    };

    let config_key = Pubkey::find_program_address(&[b"config"], &nft_bridge_program_id).0;
    let custody_key =
        Pubkey::find_program_address(&[input.mint.as_ref()], &nft_bridge_program_id).0;

    let authority_signer =
        Pubkey::find_program_address(&[b"authority_signer"], &nft_bridge_program_id).0;

    let custody_signer =
        Pubkey::find_program_address(&[b"custody_signer"], &nft_bridge_program_id).0;

    // SPL Metadata
    let spl_metadata = Pubkey::find_program_address(
        &[
            b"metadata".as_ref(),
            mpl_token_metadata::id().as_ref(),
            input.mint.as_ref(),
        ],
        &mpl_token_metadata::id(),
    )
    .0;

    let emitter = Pubkey::find_program_address(&[b"emitter"], &nft_bridge_program_id).0;

    let bridge_config = Pubkey::find_program_address(&[b"Bridge"], &wormhole_core_program_id).0;

    let sequence =
        Pubkey::find_program_address(&[b"Sequence", emitter.as_ref()], &wormhole_core_program_id).0;

    let fee_collector =
        Pubkey::find_program_address(&[b"fee_collector"], &wormhole_core_program_id).0;

    // TODO: use a real nonce
    let nonce = rand::thread_rng().gen();

    let wrapped_data = TransferNativeData {
        nonce,
        target_address: input.target_address.0,
        target_chain: input.target_chain,
    };

    let ix = solana_program::instruction::Instruction {
        program_id: nft_bridge_program_id,
        accounts: vec![
            AccountMeta::new(input.payer.pubkey(), true),
            AccountMeta::new_readonly(config_key, false),
            AccountMeta::new(input.from, false),
            AccountMeta::new(input.mint, false),
            AccountMeta::new_readonly(spl_metadata, false),
            AccountMeta::new(custody_key, false),
            AccountMeta::new_readonly(authority_signer, false),
            AccountMeta::new_readonly(custody_signer, false),
            AccountMeta::new(bridge_config, false),
            AccountMeta::new(input.message.pubkey(), true),
            AccountMeta::new_readonly(emitter, false),
            AccountMeta::new(sequence, false),
            AccountMeta::new(fee_collector, false),
            AccountMeta::new_readonly(solana_program::sysvar::clock::id(), false),
            // Dependencies
            AccountMeta::new_readonly(solana_program::sysvar::rent::id(), false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            // Program
            AccountMeta::new_readonly(wormhole_core_program_id, false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
        data: (NFTBridgeInstructions::TransferWrapped, wrapped_data).try_to_vec()?,
    };

    let minimum_balance_for_rent_exemption = ctx
        .solana_client
        .get_minimum_balance_for_rent_exemption(std::mem::size_of::<
            mpl_bubblegum::accounts::CreateTree,
        >())
        .await?;

    let ins = Instructions {
        fee_payer: input.payer.pubkey(),
        signers: [input.payer.clone_keypair(), input.message.clone_keypair()].into(),
        instructions: [ix].into(),
        minimum_balance_for_rent_exemption,
    };

    let ins = input.submit.then_some(ins).unwrap_or_default();

    let signature = ctx
        .execute(
            ins,
            value::map! {
                "spl_metadata" => spl_metadata,
            },
        )
        .await?
        .signature;

    Ok(Output { signature })
}
