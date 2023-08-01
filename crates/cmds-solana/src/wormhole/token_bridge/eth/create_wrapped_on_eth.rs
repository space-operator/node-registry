use core::panic;
use std::str::FromStr;

use crate::{
    prelude::*,
    wormhole::token_bridge::eth::{CreateWrappedResponse, Response as ServerlessOutput},
};

use borsh::BorshSerialize;
use serde_json::json;
use solana_program::instruction::AccountMeta;
use solana_sdk::pubkey::Pubkey;

// Command Name
const NAME: &str = "create_wrapped_on_eth";

const DEFINITION: &str = include_str!(
    "../../../../../../node-definitions/solana/wormhole/token_bridge/eth/create_wrapped_on_eth.json"
);

fn build() -> Result<Box<dyn CommandTrait>, CommandError> {
    use once_cell::sync::Lazy;
    static CACHE: Lazy<Result<CmdBuilder, BuilderError>> =
        Lazy::new(|| CmdBuilder::new(DEFINITION)?.check_name(NAME));

    Ok(CACHE.clone()?.build(run))
}

inventory::submit!(CommandDescription::new(NAME, |_| { build() }));

#[derive(Serialize, Deserialize, Debug)]
pub struct Input {
    pub keypair: String,
    pub network_name: String,
    pub signed_vaa: String,
    pub token: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    response: CreateWrappedResponse,
}

async fn run(mut ctx: Context, input: Input) -> Result<Output, CommandError> {
    #[derive(Serialize, Deserialize, Debug)]
    struct Payload {
        #[serde(rename = "networkName")]
        network_name: String,
        keypair: String,
        #[serde(rename = "signedVAA")]
        signed_vaa: String,
        token: String,
    }

    let payload = Payload {
        network_name: input.network_name,
        keypair: input.keypair,
        signed_vaa: input.signed_vaa,
        token: input.token,
    };

    let client = reqwest::Client::new();
    let response: CreateWrappedResponse = client
        .post("https://gygvoikm3c.execute-api.us-east-1.amazonaws.com/create_wrapped_on_eth")
        .json(&payload)
        .send()
        .await?
        .json::<CreateWrappedResponse>()
        .await?;

    Ok(Output { response })
}

#[cfg(test)]
mod tests {
    use crate::wormhole::token_bridge::eth::CreateWrappedResponse;

    use super::*;

    #[derive(Serialize, Deserialize, Debug)]
    struct Payload {
        #[serde(rename = "networkName")]
        network_name: String,
        keypair: String,
        #[serde(rename = "signedVAA")]
        signed_vaa: String,
        token: String,
    }

    #[tokio::test]
    async fn test_local() {
        async fn test(payload: Payload) -> Result<CreateWrappedResponse, reqwest::Error> {
            let client = reqwest::Client::new();
            let json = client
                .post(
                    "https://gygvoikm3c.execute-api.us-east-1.amazonaws.com/create_wrapped_on_eth",
                )
                .json(&payload)
                .send()
                .await?
                .json::<serde_json::Value>()
                .await?;

            dbg!(&json);

            let response = serde_json::from_value(json).unwrap();

            Ok(response)
        }

        let payload = Payload {
            network_name: "devnet".into(),
            keypair: "".into(),
            signed_vaa: "AQAAAAABACTH+5e/0/FZYU/YqK3hYxfoR0vdBK5O4yAveiILfgbQDrhVmE1r6jFD9tNzLmC4npm2iBMs8yWEkYiw4QyCTVEBZMlzE++C5sQAATsmQJ+Kre0/XdyhhGlapqD6gpsMhcr4SFYySJbSFMqYAAAAAAAAX9kgApi7fW9jhJ4tQJVc/RdjCHlBXnAU+DA652cYv7j3QvzCAAEJAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==".into(),
       token:"".into()
        };

        let res = test(payload).await.unwrap();
        dbg!(res);
        std::panic!()
    }
}
