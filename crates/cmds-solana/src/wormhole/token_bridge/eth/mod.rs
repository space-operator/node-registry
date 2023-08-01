use serde::{Deserialize, Serialize};

pub mod attest_from_eth;
pub mod create_wrapped_on_eth;
pub mod transfer_from_eth;

#[derive(Serialize, Deserialize, Debug)]
struct GasUsed {
    #[serde(rename = "type")]
    gas_type: String,
    hex: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct EffectiveGasPrice {
    #[serde(rename = "type")]
    gas_type: String,
    hex: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Receipt {
    to: String,
    from: String,
    contract_address: Option<String>,
    #[serde(rename = "transactionIndex")]
    transaction_index: u32,
    #[serde(rename = "gasUsed")]
    gas_used: GasUsed,
    #[serde(rename = "logsBloom")]
    logs_bloom: String,
    #[serde(rename = "blockHash")]
    block_hash: String,
    #[serde(rename = "transactionHash")]
    transaction_hash: String,
    logs: Vec<Log>,
    #[serde(rename = "blockNumber")]
    block_number: u32,
    confirmations: u32,
    #[serde(rename = "cumulativeGasUsed")]
    cumulative_gas_used: GasUsed,
    #[serde(rename = "effectiveGasPrice")]
    effective_gas_price: EffectiveGasPrice,
    status: u32,
    r#type: u32,
    byzantium: bool,
    events: Vec<Log>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Log {
    #[serde(rename = "transactionIndex")]
    transaction_index: u32,
    #[serde(rename = "blockNumber")]
    block_number: u32,
    #[serde(rename = "transactionHash")]
    transaction_hash: String,
    address: String,
    topics: Vec<String>,
    data: String,
    #[serde(rename = "logIndex")]
    log_index: u32,
    #[serde(rename = "blockHash")]
    block_hash: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Output {
    receipt: Receipt,
    #[serde(rename = "emitterAddress")]
    emitter_address: String,
    sequence: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Response {
    output: Output,
}

#[derive(Serialize, Deserialize, Debug)]
struct CreateWrappedOutput {
    receipt: Receipt,
}

#[derive(Serialize, Deserialize, Debug)]
struct CreateWrappedResponse {
    output: CreateWrappedOutput,
    address: String,
}
