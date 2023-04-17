use crate::prelude::*;
use flow_lib::command::builder::{BuildResult, BuilderCache};
use hyper::client::connect::dns::Name as DomainName;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::{net::SocketAddr, str::FromStr};

const HTTP_REQUEST: &str = "http_request";

fn build() -> BuildResult {
    const CACHE: BuilderCache = BuilderCache::new(|| {
        CmdBuilder::new(include_str!("../../../node-definitions/http.json"))?
            .check_name(HTTP_REQUEST)
    });
    Ok(CACHE.clone()?.build(run))
}

inventory::submit!(CommandDescription::new(HTTP_REQUEST, |_| build()));

fn default_method() -> String {
    "GET".to_owned()
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Input {
    pub url: String,
    #[serde(default = "default_method")]
    pub method: String,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub query_params: HashMap<String, String>,
    #[serde(default)]
    pub body: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    body: Value,
    headers: HashMap<String, String>,
}

struct Resolver;

impl reqwest::dns::Resolve for Resolver {
    fn resolve(&self, name: DomainName) -> reqwest::dns::Resolving {
        Box::pin(async move {
            tracing::debug!("resolving {}", name.as_str());
            let host = name.as_str().to_owned() + ":0";

            let addrs = tokio::net::lookup_host(host).await?;
            let addrs: Box<dyn Iterator<Item = SocketAddr> + Send> = Box::new(addrs);
            Ok(addrs)
        })
    }
}

async fn run(_: Context, input: Input) -> Result<Output, CommandError> {
    let client = reqwest::Client::builder()
        .dns_resolver(Arc::new(Resolver))
        .build()?;

    let mut req = client.request(input.method.parse()?, &input.url);

    if !input.query_params.is_empty() {
        req = req.query(&input.query_params);
    }

    if !input.headers.is_empty() {
        let headers = input
            .headers
            .iter()
            .map(|(k, v)| {
                let name = HeaderName::from_str(&k)?;
                let value = HeaderValue::from_str(&v)?;
                Ok((name, value))
            })
            .collect::<Result<HeaderMap, CommandError>>()?;
        req = req.headers(headers);
    }

    if let Some(body) = input.body {
        req = req.json(&body);
    }

    let resp = req.send().await?;

    let status = resp.status();

    if status.is_success() {
        let headers = resp
            .headers()
            .iter()
            .map(|(k, v)| {
                (
                    k.as_str().to_lowercase(),
                    String::from_utf8_lossy(v.as_bytes()).into_owned(),
                )
            })
            .collect::<HashMap<String, String>>();

        let ct = headers
            .get("content-type")
            .map(String::as_str)
            .unwrap_or("text/plain");
        let body: Value = if ct.starts_with("text/") {
            resp.text().await?.into()
        } else if ct.contains("json") {
            resp.json::<serde_json::Value>().await?.into()
        } else {
            resp.bytes().await?.into()
        };

        Ok(Output { headers, body })
    } else {
        let body = resp.text().await.ok();
        Err(anyhow::anyhow!(
            "status code: {}\n{}",
            status.as_u16(),
            body.unwrap_or_default()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build() {
        build().unwrap();
    }
}
