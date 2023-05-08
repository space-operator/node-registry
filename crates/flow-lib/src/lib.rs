pub mod command;
pub mod config;
pub mod context;
pub mod solana;
pub mod utils;

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

pub type UserId = uuid::Uuid;

pub use config::{
    CmdInputDescription, CmdOutputDescription, CommandType, ContextConfig, FlowConfig, FlowId,
    FlowRunId, Gate, HttpClientConfig, Name, NodeConfig, NodeId, SolanaClientConfig, SolanaNet,
    ValueSet, ValueType,
};
pub use context::{Context, User};
pub use value::{self, Error as ValueError, Value};
