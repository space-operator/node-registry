pub mod command;
pub mod config;
pub mod context;

pub use config::{
    CmdInputDescription, CmdOutputDescription, CommandType, ContextConfig, FlowConfig, FlowId,
    FlowRunId, Gate, HttpClientConfig, Name, NodeConfig, NodeId, SolanaClientConfig, SolanaNet,
    ValueSet, ValueType,
};
pub use context::{Context, User};
pub use value::{Error as ValueError, Value};

pub mod solana {
    use solana_sdk::{instruction::Instruction, pubkey::Pubkey, signer::keypair::Keypair};

    #[derive(Default)]
    pub struct Instructions {
        pub fee_payer: Pubkey,
        pub signers: std::collections::BTreeSet<Keypair>,
        pub minimum_balance_for_rent_exemption: u64,
        pub instructions: Vec<Instruction>,
    }
}
