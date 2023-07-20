use std::str::FromStr;

use crate::prelude::*;

use borsh::BorshSerialize;
use solana_program::instruction::AccountMeta;
use solana_sdk::pubkey::Pubkey;

use super::TokenBridgeInstructions;

// Command Name
const NAME: &str = "initialize_token_bridge";

const DEFINITION: &str =
    include_str!("../../../../../node-definitions/solana/wormhole/token_bridge/initialize.json");

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

    let token_bridge_program_id = match ctx.cfg.solana_client.cluster {
        SolanaNet::Mainnet => Pubkey::from_str("wormDTUJ6AWPNvk59vGQbDvGJmqbDTdgWgAqcLBCgUb")?,
        // TODO testnet not deployed yet
        SolanaNet::Testnet => Pubkey::from_str("DZnkkTmCiFWfYTfT41X3Rd1kDgozqzxWaHqsw6W4x2oe")?,
        SolanaNet::Devnet => Pubkey::from_str("DZnkkTmCiFWfYTfT41X3Rd1kDgozqzxWaHqsw6W4x2oe")?,
    };

    let config_key = Pubkey::find_program_address(&[b"config"], &token_bridge_program_id).0;

    let ix = solana_program::instruction::Instruction {
        program_id: token_bridge_program_id,
        accounts: vec![
            AccountMeta::new(input.payer.pubkey(), true),
            AccountMeta::new(config_key, false),
            // Dependencies
            AccountMeta::new(solana_program::sysvar::rent::id(), false),
            AccountMeta::new(solana_program::system_program::id(), false),
        ],
        data: (
            TokenBridgeInstructions::Initialize,
            wormhole_core_program_id,
        )
            .try_to_vec()?,
    };

    //FIXME fix all sizes
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

    let signature = ctx.execute(ins, <_>::default()).await?.signature;

    Ok(Output { signature })
}
