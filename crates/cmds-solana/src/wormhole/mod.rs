use anchor_lang::AnchorSerialize;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

pub mod get_vaa;
pub mod parse_vaa;
pub mod post_message;

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

#[derive(Debug, Deserialize, Serialize, Clone)]
struct WormholeResponse {
    data: Vec<WormholeData>,
    pagination: WormholePagination,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct WormholeData {
    sequence: u64,
    id: String,
    version: u64,
    emitter_chain: u64,
    emitter_addr: String,
    emitter_native_addr: String,
    guardian_set_index: u64,
    vaa: String,
    timestamp: String,
    updated_at: String,
    indexed_at: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct WormholePagination {
    next: String,
}

// // Structs for API VAA parsing
// #[derive(Serialize, Deserialize, Debug)]
// struct GuardianSignature {
//     index: u8,
//     signature: Vec<u8>,
// }

// #[derive(Serialize, Deserialize, Debug)]
// struct ParsedVaa {
//     version: u8,
//     guardian_set_index: u32,
//     guardian_signatures: Vec<GuardianSignature>,
//     timestamp: u32,
//     nonce: u32,
//     emitter_chain: u16,
//     emitter_address: [u8; 32],
//     sequence: u64,
//     consistency_level: u8,
//     payload: Vec<u8>,
// }

