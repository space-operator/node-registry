pub mod associated_token_account;
pub mod clockwork;
pub mod create_mint_account;
pub mod create_token_account;
pub mod error;
pub mod find_pda;
pub mod generate_keypair;
pub mod get_balance;
pub mod metaboss;
pub mod mint_token;
pub mod nft;
pub mod proxy_authority;
pub mod request_airdrop;
pub mod request_http;
pub mod std;
pub mod transfer_sol;
pub mod transfer_token;
pub mod utils;
pub mod wallet;

pub use error::{Error, Result};

pub mod prelude {
    pub use crate::utils::{execute, submit_transaction, try_sign_wallet, KeypairExt};
    pub use async_trait::async_trait;
    pub use flow_lib::{
        command::{CommandDescription, CommandError, CommandTrait},
        context::Context,
        CmdInputDescription as CmdInput, CmdOutputDescription as CmdOutput, Name, SolanaNet,
        ValueSet, ValueType,
    };
    pub use rust_decimal::Decimal;
    pub use serde::{Deserialize, Serialize};
    pub use solana_client::nonblocking::rpc_client::RpcClient;
    pub use solana_sdk::{
        pubkey::Pubkey,
        signature::{Keypair, Signature},
        signer::Signer,
    };
    pub use std::sync::Arc;
    pub use value::{HashMap, Value};
}

#[cfg(test)]
pub mod tests {
    use crate::prelude::*;
    /*
    use flow_lib::{
        config::client::{NodeData, TargetsForm},
        CommandType,
    };

    #[test]
    fn test_input_convention() {
        let data = NodeData {
            r#type: CommandType::Native,
            node_id: String::new(),
            sources: Vec::new(),
            targets: Vec::new(),
            targets_form: TargetsForm {
                form_data: Default::default(),
                extra: Default::default(),
                wasm_bytes: None,
            },
        };
        for CommandDescription { name, fn_new } in inventory::iter::<CommandDescription>() {
            let c = (fn_new)(&data);
            for input in c.inputs() {
                if input.type_bounds[0] == ValueType::Pubkey {
                    // pubkey input should also accept keypair and string
                    assert!(
                        input.type_bounds.contains(&ValueType::Keypair),
                        "pubkey: {name}, {}",
                        input.name
                    );
                    assert!(
                        input.type_bounds.contains(&ValueType::String),
                        "pubkey: {name}, {}",
                        input.name
                    );
                } else if input.type_bounds[0] == ValueType::Keypair {
                    // keypair input should also accept string
                    assert!(
                        input.type_bounds.contains(&ValueType::String),
                        "keypair: {name}, {}",
                        input.name
                    );
                }
            }
        }
    }
    */

    #[test]
    fn test_name_unique() {
        let mut m = std::collections::HashSet::new();
        let mut dup = false;
        for CommandDescription { name, .. } in inventory::iter::<CommandDescription>() {
            if !m.insert(name) {
                println!("Dupicated: {}", name);
                dup = true;
            }
        }
        assert!(!dup);
    }
}
