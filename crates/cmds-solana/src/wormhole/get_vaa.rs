use serde_json::Value as JsonValue;
use serde_wormhole::RawMessage;
use std::str::FromStr;

use crate::{
    prelude::*,
    wormhole::{WormholeInstructions, WormholeResponse},
};

use super::PostMessageData;
use borsh::{BorshDeserialize, BorshSerialize};
use rand::Rng;
use reqwest::Error;
use solana_program::{instruction::AccountMeta, system_instruction, sysvar};
use solana_sdk::pubkey::Pubkey;
use std::time::Duration;
use tokio::time::sleep;

use wormhole_sdk::{
    vaa::{digest, Body},
    Vaa,
};
// Command Name
const NAME: &str = "get_vaa";

const DEFINITION: &str = include_str!("../../../../node-definitions/solana/wormhole/get_vaa.json");

fn build() -> Result<Box<dyn CommandTrait>, CommandError> {
    use once_cell::sync::Lazy;
    static CACHE: Lazy<Result<CmdBuilder, BuilderError>> =
        Lazy::new(|| CmdBuilder::new(DEFINITION)?.check_name(NAME));
    Ok(CACHE.clone()?.build(run))
}

inventory::submit!(CommandDescription::new(NAME, |_| { build() }));

#[derive(Serialize, Deserialize, Debug)]
pub struct Input {
    #[serde(with = "value::pubkey")]
    pub emitter: Pubkey,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    response: Option<WormholeResponse>,
    vaa: Option<String>,
}

async fn run(mut ctx: Context, input: Input) -> Result<Output, CommandError> {
    let wormhole_endpoint = match ctx.cfg.solana_client.cluster {
        SolanaNet::Mainnet => "",
        SolanaNet::Testnet => "",
        SolanaNet::Devnet => "https://api.testnet.wormscan.io",
    }
    .to_owned();

    let wormhole_path: &str = "/api/v1/vaas";

    let chain_id = "1";

    let wormhole_url = wormhole_endpoint
        + wormhole_path
        + "/"
        + chain_id
        + "/"
        + input.emitter.to_string().as_str();

    let client = reqwest::Client::new();

    async fn send_wormhole_request(
        client: &reqwest::Client,
        wormhole_url: &str,
        timeout: Duration,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let response = client.get(wormhole_url).timeout(timeout).send().await?;
        Ok(response)
    }

    let timeout = Duration::from_secs(60);

    let mut response = send_wormhole_request(&client, &wormhole_url, timeout).await?;

    while response.status() != 200 {
        println!("Waiting for VAA to be generated...");
        sleep(Duration::from_secs(5)).await;
        response = send_wormhole_request(&client, &wormhole_url, timeout).await?;
    }

    let response_text = response.text().await?;
    let response: WormholeResponse = serde_json::from_str(&response_text)?;

    let vaa = &response.data[0].vaa;

    Ok(Output {
        response: Some(response.clone()),
        vaa: Some(vaa.to_owned()),
    })
}

#[test]
fn test() {
    use base64::decode;
    use hex::FromHex;
    use std::convert::TryInto;

    let vaa_string: &str = "AQAAAAABAIxhSgUuacMANc852yONRP8gdgED5nicWYPzM3TYaSSJD2iFwSoUTkg0yHu+hgbBHCgtYOv2VQptxuF53hobrwsAZK8aJ4yiZZ8AAWlkPNgd1d6rFUb+syWqmwfKcK3Ki+zbCl5KrF4oqa5RAAAAAAAAAAABAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";

    let vaa_bytes = decode(vaa_string).unwrap();
    let vaa_hex = hex::encode(vaa_bytes);
    dbg!(vaa_hex);

    #[derive(Debug)]
    struct GuardianSignature {
        index: u8,
        signature: [u8; 65],
    }

    #[derive(Debug)]
    struct ParsedVaa {
        version: u8,
        guardian_set_index: u32,
        guardian_signatures: Vec<GuardianSignature>,
        timestamp: u32,
        nonce: u32,
        emitter_chain: u16,
        emitter_address: [u8; 32],
        sequence: u64,
        consistency_level: u8,
        payload: Vec<u8>,
    }

    fn parse_vaa(vaa: &[u8]) -> ParsedVaa {
        let sig_start = 6;
        let num_signers = vaa[5] as usize;
        let sig_length = 66;

        let mut guardian_signatures = Vec::new();
        for i in 0..num_signers {
            let start = sig_start + i * sig_length;
            let mut signature = [0u8; 65];
            signature.copy_from_slice(&vaa[start + 1..start + 66]);
            guardian_signatures.push(GuardianSignature {
                index: vaa[start],
                signature,
            });
        }

        let body = &vaa[sig_start + sig_length * num_signers..];

        ParsedVaa {
            version: vaa[0],
            guardian_set_index: u32::from_be_bytes(vaa[1..5].try_into().unwrap()),
            guardian_signatures,
            timestamp: u32::from_be_bytes(body[0..4].try_into().unwrap()),
            nonce: u32::from_be_bytes(body[4..8].try_into().unwrap()),
            emitter_chain: u16::from_be_bytes(body[8..10].try_into().unwrap()),
            emitter_address: body[10..42].try_into().unwrap(),
            sequence: u64::from_be_bytes(body[42..50].try_into().unwrap()),
            consistency_level: body[50],
            payload: body[51..].to_vec(),
        }
    }

    let parse = parse_vaa(&decode(vaa_string).unwrap());
    dbg!(parse);
}
