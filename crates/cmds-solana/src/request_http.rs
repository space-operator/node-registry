use anyhow::bail;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::str::FromStr;
use value::from_value;

use crate::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestHttp;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Method {
    CONNECT,
    DELETE,
    GET,
    HEAD,
    OPTIONS,
    PATCH,
    POST,
    PUT,
    TRACE,
}

impl From<&str> for Method {
    fn from(method: &str) -> Self {
        let method = method.to_lowercase();
        match method.as_str() {
            "connect" => Method::CONNECT,
            "delete" => Method::DELETE,
            "get" => Method::GET,
            "head" => Method::HEAD,
            "options" => Method::OPTIONS,
            "patch" => Method::PATCH,
            "post" => Method::POST,
            "put" => Method::PUT,
            "trace" => Method::TRACE,
            _ => {
                tracing::error!("unsupported method: {}, using GET", method);
                Method::GET
            }
        }
    }
}

impl From<Method> for reqwest::Method {
    fn from(method: Method) -> reqwest::Method {
        match method {
            Method::CONNECT => reqwest::Method::CONNECT,
            Method::DELETE => reqwest::Method::DELETE,
            Method::GET => reqwest::Method::GET,
            Method::HEAD => reqwest::Method::HEAD,
            Method::OPTIONS => reqwest::Method::OPTIONS,
            Method::PATCH => reqwest::Method::PATCH,
            Method::POST => reqwest::Method::POST,
            Method::PUT => reqwest::Method::PUT,
            Method::TRACE => reqwest::Method::TRACE,
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
    pub auth: Option<Auth>,
    pub headers: Option<HashMap<String, String>>,
    pub query_params: Option<HashMap<String, String>>,
    pub body: Option<Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Outputs {
    headers: HashMap<String, String>,
    body: Option<Value>,
}

// Command Name
const HTTP_REQUEST: &str = "http_request";

// Inputs
const METHOD: &str = "method";
const URL: &str = "url";
const AUTH: &str = "auth";
const HEADERS: &str = "headers";
const QUERY_PARAMS: &str = "query_params";
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
                required: false,
                passthrough: false,
            },
            CmdInput {
                name: HEADERS.into(),
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
                name: HEADERS.into(),
                r#type: ValueType::Json,
            },
        ]
        .to_vec()
    }

    async fn run(&self, _ctx: Context, inputs: ValueSet) -> Result<ValueSet, CommandError> {
        let inputs: Inputs = value::from_map(inputs)?;

        let client = reqwest::Client::new();

        // Build request
        let mut req = client.request(inputs.method.into(), inputs.url.clone());

        if let Some(query_params) = &inputs.query_params {
            let query = query_params
                .iter()
                .map(|(field_name, field_value)| (field_name, field_value))
                .collect::<HashMap<_, _>>();
            req = req.query(&query);
        }

        if let Some(headers) = &inputs.headers {
            let headers = headers
                .iter()
                .flat_map(|(header_name, header_value)| {
                    let header_name = HeaderName::from_str(header_name).map_err(|_| {
                        anyhow::anyhow!("InvalidHttpHeaderNameInput: {}", header_name.clone())
                    })?;
                    let header_value = HeaderValue::from_str(header_value).map_err(|_| {
                        anyhow::anyhow!("InvalidHttpHeaderValueInput: {}", header_value)
                    })?;
                    Ok::<(HeaderName, HeaderValue), CommandError>((header_name, header_value))
                })
                .collect::<HeaderMap>();
            req = req.headers(headers);
        }

        if let Some(auth) = &inputs.auth {
            match auth {
                Auth::Basic { username, password } => {
                    req = req.basic_auth(username, password.as_ref());
                }
                Auth::Bearer(token) => {
                    req = req.bearer_auth(token);
                }
                Auth::NoAuth => (),
            }
        }

        if let Some(body) = inputs.body {
            let body: serde_json::Value = from_value(body)?;
            req = req.json(&body);
        }

        // Run the request
        let resp = req
            .send()
            .await
            .map_err(|_| anyhow::anyhow!("HttpRequestError"))?;

        // Check the status
        let status = resp.status();

        if !status.is_success() {
            let body = resp.text().await.ok();
            bail!(
                "HttpStatusError: {}: {}",
                status.as_u16(),
                body.unwrap_or_default()
            );
        }

        // Build outputs
        let mut output = Outputs::default();

        let headers = resp.headers();

        for (header_name, _field_name) in headers.iter() {
            let header_val = headers
                .get(header_name)
                .ok_or_else(|| anyhow::anyhow!("HttpHeaderNotFound: {}", header_name.clone()))?;
            let header_val = header_val
                .to_str()
                .map_err(|_| {
                    anyhow::anyhow!("InvalidHttpHeaderValueOutput: {}", header_name.clone())
                })?
                .into();
            output.headers.insert(header_name.to_string(), header_val);
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|_| anyhow::anyhow!("ReadHttpJsonBody"))?;

        output.body = Some(Value::from(body));

        // Return outputs
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
                    auth: Some(Auth::NoAuth),
                    query_params: Some(query_params),
                    body: None,
                    headers: None,
                })
                .unwrap(),
            )
            .await
            .unwrap();

        let output: Outputs = value::from_map(output).unwrap();
        dbg!(&output);
        let body = output.body.unwrap();
        let price = value::crud::get(&body, &["ethereum", "usd"]).unwrap();
        dbg!(price);
    }
}
