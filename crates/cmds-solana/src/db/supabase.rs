use hyper::header::HeaderValue;
use reqwest::Body;
use solana_program::blake3::Hash;
use tracing_log::log::info;

use crate::prelude::*;

// Command Name
const NAME: &str = "supabase";

const DEFINITION: &str = include_str!("../../../../node-definitions/db/supabase.json");

fn build() -> Result<Box<dyn CommandTrait>, CommandError> {
    use once_cell::sync::Lazy;
    static CACHE: Lazy<Result<CmdBuilder, BuilderError>> =
        Lazy::new(|| CmdBuilder::new(DEFINITION)?.check_name(NAME));
    Ok(CACHE.clone()?.build(run))
}

inventory::submit!(CommandDescription::new(NAME, |_| { build() }));

#[derive(Serialize, Deserialize, Debug)]
pub struct Input {
    pub string: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    pub res: HashMap<String, String>,
}

async fn run(mut ctx: Context, input: Input) -> Result<Output, CommandError> {
    info!("{:#?}", ctx.environment);

    info!("{:#?}", ctx.user.id);
    let bearer = ctx.environment.get("authorization_bearer").unwrap();
    let apikey = ctx.environment.get("apikey").unwrap();

    // use reqwest to make a request to supabase
    let client = reqwest::Client::new();

    // headers
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        "authorization_bearer",
        HeaderValue::from_str(&bearer).unwrap(),
    );
    headers.insert("apikey", HeaderValue::from_str(&apikey).unwrap());

    let res = client
        .post("https://hyjboblkjeevkzaqsyxe.supabase.co/rest/v1/users_nft")
        .headers(headers)
        .json(&input.string)
        .send()
        .await?
        .json::<HashMap<String, String>>()
        .await?;

    // https://hyjboblkjeevkzaqsyxe.supabase.co/rest/v1/users_nft?id=eq.1&select=*
    Ok(Output { res })
}
