use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

pub mod post_message;

use anchor_lang::prelude::*;

#[repr(u8)]
#[derive(BorshSerialize, BorshDeserialize)]
pub enum WormholeInstructions {
    Initialize,
    PostMessage,
    PostVAA,
    SetFees,
    TransferFees,
    UpgradeContract,
    UpgradeGuardianSet,
    VerifySignatures,
    PostMessageUnreliable,
}

#[derive(AnchorSerialize, Deserialize, Serialize)]
pub struct PostMessageData {
    /// Unique nonce for this message
    pub nonce: u32,

    /// Message payload
    pub payload: Vec<u8>,

    /// Commitment Level required for an attestation to be produced
    pub consistency_level: ConsistencyLevel,
}

#[repr(u8)]
#[derive(AnchorSerialize, Clone, Serialize, Deserialize)]
pub enum ConsistencyLevel {
    Confirmed,
    Finalized,
}
