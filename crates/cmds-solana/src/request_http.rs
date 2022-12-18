use std::str::FromStr;

use anyhow::bail;
use log::error;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

use crate::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestHttp;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Method {
    Connect,
    Delete,
    Get,
    Head,
    Options,
    Patch,
    Post,
    Put,
    Trace,
}

impl From<&str> for Method {
    fn from(method: &str) -> Self {
        let method = method.to_lowercase();
        match method.as_str() {
            "connect" => Method::Connect,
            "delete" => Method::Delete,
            "get" => Method::Get,
            "head" => Method::Head,
            "options" => Method::Options,
            "patch" => Method::Patch,
            "post" => Method::Post,
            "put" => Method::Put,
            "trace" => Method::Trace,
            _ => {
                error!("Unsupported method: {}; Using normal Get", method);
                Method::Get
            }
        }
    }
}

impl From<Method> for reqwest::Method {
    fn from(method: Method) -> reqwest::Method {
        match method {
            Method::Connect => reqwest::Method::CONNECT,
            Method::Delete => reqwest::Method::DELETE,
            Method::Get => reqwest::Method::GET,
            Method::Head => reqwest::Method::HEAD,
            Method::Options => reqwest::Method::OPTIONS,
            Method::Patch => reqwest::Method::PATCH,
            Method::Post => reqwest::Method::POST,
            Method::Put => reqwest::Method::PUT,
            Method::Trace => reqwest::Method::TRACE,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Auth {
    Basic {
        username: String,
        password: Option<String>,
    },
    Bearer(String),
    NoAuth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inputs {
    pub method: Method,
    pub url: String,
    pub auth: Auth,
    pub input_headers: HashMap<String, String>,
    pub output_headers: HashMap<String, String>,
    pub input_query_params: HashMap<String, String>,
    pub body: Option<Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Outputs {
    output_headers: HashMap<String, String>,
    body: Option<Value>,
}

const HTTP_REQUEST: &str = "http_request";

// Inputs
const METHOD: &str = "method";
const URL: &str = "url";
const AUTH: &str = "auth";
const INPUT_HEADERS: &str = "input_headers";
const OUTPUT_HEADERS: &str = "output_headers";
const QUERY_PARAMS: &str = "input_query_params";
const BODY: &str = "body";

#[async_trait]
impl CommandTrait for RequestHttp {
    fn name(&self) -> Name {
        HTTP_REQUEST.into()
    }

    fn inputs(&self) -> Vec<CmdInput> {
        [
            CmdInput {
                name: METHOD.into(),
                type_bounds: [ValueType::String].to_vec(),
                required: true,
                passthrough: false,
            },
            CmdInput {
                name: URL.into(),
                type_bounds: [ValueType::String].to_vec(),
                required: true,
                passthrough: false,
            },
            CmdInput {
                name: AUTH.into(),
                type_bounds: [ValueType::Json].to_vec(),
                required: true,
                passthrough: false,
            },
            CmdInput {
                name: INPUT_HEADERS.into(),
                type_bounds: [ValueType::Json].to_vec(),
                required: false,
                passthrough: false,
            },
            CmdInput {
                name: OUTPUT_HEADERS.into(),
                type_bounds: [ValueType::Json].to_vec(),
                required: false,
                passthrough: false,
            },
            CmdInput {
                name: QUERY_PARAMS.into(),
                type_bounds: [ValueType::Json].to_vec(),
                required: false,
                passthrough: false,
            },
            CmdInput {
                name: BODY.into(),
                type_bounds: [ValueType::Json].to_vec(),
                required: false,
                passthrough: false,
            },
        ]
        .to_vec()
    }

    fn outputs(&self) -> Vec<CmdOutput> {
        [
            CmdOutput {
                name: BODY.into(),
                r#type: ValueType::Json,
            },
            CmdOutput {
                name: OUTPUT_HEADERS.into(),
                r#type: ValueType::Pubkey,
            },
        ]
        .to_vec()
    }

    async fn run(&self, _ctx: Context, inputs: ValueSet) -> Result<ValueSet, CommandError> {
        let inputs: Inputs = value::from_map(inputs)?;
        let client = reqwest::Client::new();
        let mut req = client.request(inputs.method.into(), inputs.url.clone());
        if !inputs.input_query_params.is_empty() {
            let query = inputs
                .input_query_params
                .iter()
                .map(|(field_name, field_value)| (field_name, field_value))
                .collect::<HashMap<_, _>>();
            req = req.query(&query);
        }
        if !inputs.input_headers.is_empty() {
            let headers = inputs
                .input_headers
                .iter()
                .flat_map(|(header_name, header_value)| {
                    let header_name = HeaderName::from_str(header_name).map_err(|_| {
                        anyhow::anyhow!("InvalidHttpHeaderNameInput: {}", header_name.clone())
                    })?;
                    let header_value = HeaderValue::from_str(&header_value).map_err(|_| {
                        anyhow::anyhow!("InvalidHttpHeaderValueInput: {}", header_value)
                    })?;
                    Ok::<(HeaderName, HeaderValue), CommandError>((header_name, header_value))
                })
                .collect::<HeaderMap>();
            req = req.headers(headers);
        }
        match &inputs.auth {
            Auth::Basic { username, password } => {
                req = req.basic_auth(username, password.as_ref());
            }
            Auth::Bearer(token) => {
                req = req.bearer_auth(token);
            }
            Auth::NoAuth => (),
        }
        if let Some(body) = inputs.body {
            req = req.json(&body);
        }
        let resp = req
            .send()
            .await
            .map_err(|_| anyhow::anyhow!("HttpRequestError"))?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.ok();
            bail!(
                "HttpStatusError: {}: {}",
                status.as_u16(),
                body.unwrap_or_default()
            );
        }
        let mut output = Outputs::default();
        if !inputs.output_headers.is_empty() {
            let headers = resp.headers();
            for (header_name, _field_name) in inputs.output_headers.iter() {
                let header_val = headers.get(header_name).ok_or_else(|| {
                    anyhow::anyhow!("HttpHeaderNotFound: {}", header_name.clone())
                })?;
                let header_val = header_val
                    .to_str()
                    .map_err(|_| {
                        anyhow::anyhow!("InvalidHttpHeaderValueOutput: {}", header_name.clone())
                    })?
                    .into();
                output
                    .output_headers
                    .insert(header_name.clone(), header_val);
            }
        }

        let body: Value = resp
            .json()
            .await
            .map_err(|_| anyhow::anyhow!("ReadHttpJsonBody"))?;
        output.body = Some(body.into());

        Ok(value::to_map(&output)?)
    }
}

inventory::submit!(CommandDescription::new(HTTP_REQUEST, |_| Box::new(
    RequestHttp
)));

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_valid() {
        let ctx = Context::default();

        let mut query_params = HashMap::new();
        query_params.insert("ids".into(), "ethereum".into());
        query_params.insert("vs_currencies".into(), "usd".into());

        let output = RequestHttp
            .run(
                ctx,
                value::to_map(&Inputs {
                    method: "GET".into(),
                    url: "https://api.coingecko.com/api/v3/simple/price".into(),
                    auth: Auth::NoAuth,
                    input_headers: HashMap::new(),
                    input_query_params: query_params,
                    body: None,
                    output_headers: HashMap::new(),
                })
                .unwrap(),
            )
            .await
            .unwrap();

        dbg!(&output);
        assert!(output.contains_key(BODY));
        assert!(output.contains_key(OUTPUT_HEADERS));
    }
}
