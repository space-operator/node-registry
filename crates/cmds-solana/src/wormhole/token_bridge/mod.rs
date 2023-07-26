use borsh::{BorshDeserialize, BorshSerialize};

pub mod attest;
pub mod create_wrapped;
pub mod initialize;

pub mod eth;

#[repr(u8)]
#[derive(BorshSerialize, BorshDeserialize)]
enum TokenBridgeInstructions {
    Initialize,
    AttestToken,
    CompleteNative,
    CompleteWrapped,
    TransferWrapped,
    TransferNative,
    RegisterChain,
    CreateWrapped,
    UpgradeContract,
    CompleteNativeWithPayload,
    CompleteWrappedWithPayload,
    TransferWrappedWithPayload,
    TransferNativeWithPayload,
}

#[derive(BorshDeserialize, BorshSerialize, Default)]
pub struct AttestTokenData {
    pub nonce: u32,
}

#[derive(BorshDeserialize, BorshSerialize, Default)]
pub struct CreateWrappedData {}
