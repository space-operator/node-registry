use serde_json::Value as JsonValue;
use std::str::FromStr;

use crate::{
    prelude::*,
    wormhole::{WormholeInstructions, WormholeResponse},
};

use super::PostMessageData;
use borsh::BorshSerialize;
use rand::Rng;
use reqwest::Error;
use solana_program::{instruction::AccountMeta, system_instruction, sysvar};
use solana_sdk::pubkey::Pubkey;
use std::time::Duration;
use tokio::time::sleep;

// use wormhole_anchor_sdk::wormhole;

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
    Ok(Output {
        response: Some(response),
    })
}
