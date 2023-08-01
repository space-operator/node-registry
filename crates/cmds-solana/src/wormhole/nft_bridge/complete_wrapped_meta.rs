use crate::wormhole::{PostVAAData, VAA};
use std::str::FromStr;

use crate::prelude::*;

use borsh::BorshSerialize;

use solana_program::{instruction::AccountMeta, system_program, sysvar};
use solana_sdk::pubkey::Pubkey;
use wormhole_sdk::nft::Message;

use super::{CompleteWrappedMetaData, NFTBridgeInstructions, PayloadTransfer};

// Command Name
const NAME: &str = "nft_complete_wrapped_meta";

const DEFINITION: &str = include_str!(
    "../../../../../node-definitions/solana/wormhole/nft_bridge/nft_complete_wrapped_meta.json"
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
    pub vaa: bytes::Bytes,
    pub payload: wormhole_sdk::nft::Message,
    pub vaa_hash: bytes::Bytes,
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

    let vaa =
        VAA::deserialize(&input.vaa).map_err(|_| anyhow::anyhow!("Failed to deserialize VAA"))?;
    let vaa: PostVAAData = vaa.into();

    let payload: PayloadTransfer = match input.payload {
        Message::Transfer {
            nft_address,
            nft_chain,
            symbol,
            name,
            token_id,
            uri,
            to,
            to_chain,
        } => PayloadTransfer {
            token_address: wormhole_sdk::Address::from(nft_address).0,
            token_chain: wormhole_sdk::Chain::from(nft_chain).into(),
            to: wormhole_sdk::Address::from(to).0,
            to_chain: wormhole_sdk::Chain::from(to_chain).into(),
            symbol: symbol.to_string(),
            name: name.to_string(),
            token_id: primitive_types::U256::from(token_id.0).into(),
            uri: uri.to_string(),
        },
        // ignore other arms
        _ => {
            return Err(anyhow::anyhow!("Payload content not supported"));
        }
    };

    // Convert token id
    let mut token_id = vec![0u8; 32];
    payload.token_id.to_big_endian(&mut token_id);

    let message =
        Pubkey::find_program_address(&[b"PostedVAA", &input.vaa_hash], &wormhole_core_program_id).0;

    let endpoint = Pubkey::find_program_address(
        &[
            vaa.emitter_chain.to_be_bytes().as_ref(),
            vaa.emitter_address.as_ref(),
        ],
        &nft_bridge_program_id,
    )
    .0;

    let mint = Pubkey::find_program_address(
        &[
            b"wrapped",
            payload.token_chain.to_be_bytes().as_ref(),
            payload.token_address.as_ref(),
            &token_id,
        ],
        &nft_bridge_program_id,
    )
    .0;

    let mint_meta =
        Pubkey::find_program_address(&[b"meta", mint.as_ref()], &nft_bridge_program_id).0;

    let mint_authority = Pubkey::find_program_address(&[b"mint_signer"], &nft_bridge_program_id).0;

    // SPL Metadata
    let spl_metadata = Pubkey::find_program_address(
        &[
            b"metadata".as_ref(),
            mpl_token_metadata::id().as_ref(),
            mint.as_ref(),
        ],
        &mpl_token_metadata::id(),
    )
    .0;

    let ix = solana_program::instruction::Instruction {
        program_id: nft_bridge_program_id,
        accounts: vec![
            AccountMeta::new(input.payer.pubkey(), true),
            AccountMeta::new_readonly(config_key, false),
            AccountMeta::new_readonly(message, false),
            AccountMeta::new_readonly(endpoint, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new_readonly(mint_meta, false),
            AccountMeta::new(spl_metadata, false),
            AccountMeta::new_readonly(mint_authority, false),
            // Dependencies
            AccountMeta::new_readonly(sysvar::rent::id(), false),
            AccountMeta::new_readonly(system_program::id(), false),
            // Program
            AccountMeta::new_readonly(wormhole_core_program_id, false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(spl_associated_token_account::id(), false),
            AccountMeta::new_readonly(mpl_token_metadata::id(), false),
        ],
        data: (
            NFTBridgeInstructions::CompleteWrappedMeta,
            CompleteWrappedMetaData {},
        )
            .try_to_vec()?,
    };

    let minimum_balance_for_rent_exemption = ctx
        .solana_client
        .get_minimum_balance_for_rent_exemption(std::mem::size_of::<
            mpl_bubblegum::accounts::CreateTree,
        >())
        .await?;

    let ins = Instructions {
        fee_payer: input.payer.pubkey(),
        signers: [input.payer.clone_keypair()].into(),
        instructions: [ix].into(),
        minimum_balance_for_rent_exemption,
    };

    let ins = input.submit.then_some(ins).unwrap_or_default();

    let signature = ctx
        .execute(
            ins,
            value::map! {
                "mint_metadata" => mint_meta,
                "mint" => mint,
                "spl_metadata"=> spl_metadata
            },
        )
        .await?
        .signature;

    Ok(Output { signature })
}
