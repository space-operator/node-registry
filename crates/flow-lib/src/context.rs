use crate::{ContextConfig, UserId};
use bytes::Bytes;
use solana_client::nonblocking::rpc_client::RpcClient as SolanaClient;
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use std::{any::Any, collections::HashMap, sync::Arc};
use tower::{Service, ServiceExt};

pub use http::Extensions;

pub mod signer {
    use crate::{utils::TowerClient, BoxError, UserId};
    use solana_sdk::{pubkey::Pubkey, signature::Signature};
    use thiserror::Error as ThisError;

    #[derive(ThisError, Debug)]
    pub enum Error {
        #[error("can't sign for this pubkey")]
        Pubkey,
        #[error("can't sign for this user")]
        User,
        #[error(transparent)]
        Worker(BoxError),
        #[error(transparent)]
        Other(#[from] BoxError),
    }

    pub type Svc = TowerClient<SignatureRequest, SignatureResponse, Error>;

    pub struct SignatureRequest {
        pub user_id: UserId,
        pub pubkey: Pubkey,
        pub message: bytes::Bytes,
    }

    impl actix::Message for SignatureRequest {
        type Result = Result<SignatureResponse, Error>;
    }

    pub struct SignatureResponse {
        pub signature: Signature,
    }

    pub fn unimplemented_svc() -> Svc {
        Svc::unimplemented(|| BoxError::from("unimplemented").into(), Error::Worker)
    }
}

#[derive(Clone)]
pub struct Context {
    pub cfg: ContextConfig,
    pub solana_client: Arc<SolanaClient>,
    pub environment: HashMap<String, String>,
    pub user: User,
    pub signer: signer::Svc,
    pub extensions: Arc<Extensions>,
}

impl Default for Context {
    fn default() -> Self {
        Context::from_cfg(
            &ContextConfig::default(),
            User::default(),
            signer::unimplemented_svc(),
            Extensions::default(),
        )
    }
}

#[derive(Clone, Copy)]
pub struct User {
    pub id: UserId,
    pub pubkey: Pubkey,
}

impl User {
    pub fn new(id: UserId, pubkey: [u8; 32]) -> Self {
        Self {
            id,
            pubkey: Pubkey::new_from_array(pubkey),
        }
    }
}

impl Default for User {
    /// For testing
    fn default() -> Self {
        User {
            id: uuid::uuid!("00000000-0000-0000-0000-000000000000"),
            pubkey: Pubkey::new_from_array([0u8; 32]),
        }
    }
}

impl Context {
    pub fn from_cfg(
        // TODO: pass by value
        cfg: &ContextConfig,
        user: User,
        sig_svc: signer::Svc,
        extensions: Extensions,
    ) -> Self {
        let solana_client = SolanaClient::new(cfg.solana_client.url.clone());

        Self {
            cfg: cfg.clone(),
            solana_client: Arc::new(solana_client),
            environment: cfg.environment.clone(),
            user,
            extensions: Arc::new(extensions),
            signer: sig_svc,
        }
    }

    pub async fn request_signature(
        &self,
        pubkey: Pubkey,
        message: Bytes,
    ) -> Result<Signature, anyhow::Error> {
        let mut s = self.signer.clone();
        let user_id = self.user.id;

        let signer::SignatureResponse { signature } = s
            .ready()
            .await?
            .call(signer::SignatureRequest {
                user_id,
                pubkey,
                message,
            })
            .await?;
        Ok(signature)
    }

    pub fn get<T: Any + Send + Sync + 'static>(&self) -> Option<&T> {
        self.extensions.get::<T>()
    }

    // Just a function to make sure Context is Send + Sync,
    // because !Sync will make it really hard to write async code.
    #[allow(dead_code)]
    const fn assert_send_sync() {
        const fn f<T: Send + Sync + 'static>() {}
        f::<Self>();
    }
}
