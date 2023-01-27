use crate::{ContextConfig, ValueSet};
use bytes::Bytes;
use solana_client::nonblocking::rpc_client::RpcClient as SolanaClient;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey, signature::Signature};
use std::{any::Any, collections::HashMap, sync::Arc};
use tower::{Service, ServiceExt};
use uuid::Uuid;

pub use http::Extensions;

pub mod signer {
    use solana_sdk::{pubkey::Pubkey, signature::Signature};
    use tower::{
        buffer::Buffer,
        util::{BoxService, MapErr},
    };

    pub type Svc = MapErr<
        Buffer<BoxService<SignatureRequest, SignatureResponse, anyhow::Error>, SignatureRequest>,
        fn(tower::BoxError) -> anyhow::Error,
    >;

    pub struct SignatureRequest {
        pub user_id: uuid::Uuid,
        pub pubkey: Pubkey,
        pub message: bytes::Bytes,
    }

    pub struct SignatureResponse {
        pub signature: Signature,
    }

    pub fn wrap_box(s: BoxService<SignatureRequest, SignatureResponse, anyhow::Error>) -> Svc {
        MapErr::new(Buffer::new(s, 32), map_err)
    }

    fn map_err(e: tower::BoxError) -> anyhow::Error {
        anyhow::anyhow!(e)
    }

    pub fn unimplemented_svc() -> Svc {
        let s = tower::ServiceBuilder::new()
            .boxed()
            .service_fn(|_| std::future::ready(Err(anyhow::anyhow!("not implemented"))));

        // throw away the worker
        let (s, _) = Buffer::pair(s, 32);
        MapErr::new(s, map_err)
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

#[derive(Clone)]
pub struct User {
    pub id: Uuid,
    pub pubkey: Pubkey,
}

impl User {
    pub fn new(id: Uuid, pubkey: [u8; 32]) -> Self {
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

    pub async fn send_output(
        &self,
        output: ValueSet,
        instructions: Vec<Instruction>,
    ) -> Result<Signature, anyhow::Error> {
        Err(anyhow::anyhow!("unimplemented"))
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
