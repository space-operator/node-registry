use super::KeypairExt;
use bundlr_sdk::{error::BundlrError, Ed25519Signer};
use flow_lib::Context;
use solana_sdk::signer::{keypair::Keypair, Signer};

pub struct BundlrSigner {
    keypair: Keypair,
    ctx: Context,
}

impl BundlrSigner {
    pub fn new(keypair: Keypair, ctx: Context) -> Self {
        Self { keypair, ctx }
    }
}

impl bundlr_sdk::Signer for BundlrSigner {
    const SIG_TYPE: u16 = Ed25519Signer::SIG_TYPE;
    const SIG_LENGTH: u16 = Ed25519Signer::SIG_LENGTH;
    const PUB_LENGTH: u16 = Ed25519Signer::PUB_LENGTH;

    fn sign(&self, msg: bytes::Bytes) -> Result<bytes::Bytes, BundlrError> {
        let sig = if self.keypair.is_user_wallet() {
            let rt = self
                .ctx
                .get::<tokio::runtime::Handle>()
                .ok_or_else(|| BundlrError::SigningError("tokio runtime not found".to_owned()))?
                .clone();
            let ctx = self.ctx.clone();
            let pubkey = self.keypair.pubkey();
            rt.block_on(async move {
                tokio::time::timeout(super::SIGNATURE_TIMEOUT, ctx.request_signature(pubkey, msg))
                    .await
            })
            .map_err(|e| BundlrError::SigningError(e.to_string()))?
            .map_err(|e| BundlrError::SigningError(e.to_string()))?
        } else {
            self.keypair.sign_message(&msg)
        };
        Ok(<[u8; 64]>::from(sig).to_vec().into())
    }

    fn pub_key(&self) -> bytes::Bytes {
        self.keypair.pubkey().to_bytes().to_vec().into()
    }
}
