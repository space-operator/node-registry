use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

// pub mod get_vaa;
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

#[derive(Clone, Default, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct BridgeData {
    /// The current guardian set index, used to decide which signature sets to accept.
    pub guardian_set_index: u32,

    /// Lamports in the collection account
    pub last_lamports: u64,

    /// Bridge configuration, which is set once upon initialization.
    pub config: BridgeConfig,
}

#[derive(Clone, Default, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct BridgeConfig {
    /// Period for how long a guardian set is valid after it has been replaced by a new one.  This
    /// guarantees that VAAs issued by that set can still be submitted for a certain period.  In
    /// this period we still trust the old guardian set.
    pub guardian_set_expiration_time: u32,

    /// Amount of lamports that needs to be paid to the protocol to post a message
    pub fee: u64,
}
