use crate::{prelude::*, wormhole::WormholeResponse};

use solana_sdk::pubkey::Pubkey;
use std::time::Duration;
use tokio::time::sleep;

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
    pub emitter: String,
    pub chain_id: String,
    pub sequence: String,
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

    let wormhole_url = wormhole_endpoint
        + wormhole_path
        + "/"
        + input.chain_id.as_str()
        + "/"
        + input.emitter.as_str()
        + "/"
        + input.sequence.as_str();

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
        sleep(Duration::from_secs(60)).await;
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
