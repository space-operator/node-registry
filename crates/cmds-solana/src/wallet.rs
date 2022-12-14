use crate::prelude::*;

#[derive(Debug)]
pub struct Wallet;

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    #[serde(with = "value::pubkey")]
    pub pubkey: Pubkey,
    #[serde(with = "value::keypair")]
    pub keypair: Keypair,
}

const WALLET: &str = "wallet";

#[async_trait]
impl CommandTrait for Wallet {
    fn name(&self) -> Name {
        WALLET.into()
    }

    fn inputs(&self) -> Vec<CmdInput> {
        [].to_vec()
    }

    fn outputs(&self) -> Vec<CmdOutput> {
        [
            CmdOutput {
                name: "pubkey".into(),
                r#type: ValueType::Pubkey,
            },
            CmdOutput {
                name: "keypair".into(),
                r#type: ValueType::Keypair,
            },
        ]
        .to_vec()
    }

    async fn run(&self, ctx: Context, _inputs: ValueSet) -> Result<ValueSet, CommandError> {
        let pubkey = ctx.user.pubkey;
        let mut keypair = [0u8; 64];
        keypair[32..].copy_from_slice(&pubkey.to_bytes());
        let keypair = Keypair::from_bytes(&keypair).map_err(crate::Error::custom)?;

        Ok(value::to_map(&Output { pubkey, keypair })?)
    }
}

inventory::submit!(CommandDescription::new(WALLET, |_| { Box::new(Wallet) }));

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_wallet() {
        let pubkey: Pubkey = "DKsvmM9hfNm4R94yB3VdYMZJk2ETv5hpcjuRmiwgiztY"
            .parse()
            .unwrap();

        let mut ctx = Context::default();
        ctx.user.pubkey = pubkey;

        let result = Wallet.run(ctx, Default::default()).await.unwrap();
        let output: Output = value::from_map(result).unwrap();
        assert_eq!(output.pubkey, pubkey);
        assert_eq!(output.keypair.pubkey(), pubkey);
    }
}
