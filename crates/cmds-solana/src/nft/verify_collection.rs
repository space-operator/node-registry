use crate::prelude::*;
use solana_program::instruction::Instruction;

// Command Name
const VERIFY_COLLECTION: &str = "verify_collection";

// Inputs
const MINT_ACCOUNT: &str = "mint_account";
const FEE_PAYER: &str = "fee_payer";
const COLLECTION_AUTHORITY: &str = "collection_authority";
const COLLECTION_MINT_ACCOUNT: &str = "colleciton_mint_account";
const COLLECTION_AUTHORITY_IS_DELEGATED: &str = "collection_authority_is_delegated";

// Outputs
const SIGNATURE: &str = "signature";

#[derive(Serialize, Deserialize, Debug)]
pub struct Input {
    #[serde(with = "value::pubkey")]
    pub mint_account: Pubkey,
    #[serde(with = "value::keypair")]
    pub fee_payer: Keypair,
    #[serde(with = "value::keypair")]
    pub collection_authority: Keypair,
    #[serde(with = "value::pubkey")]
    pub collection_mint_account: Pubkey,
    pub collection_authority_is_delegated: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    #[serde(with = "value::signature")]
    pub signature: Signature,
}

#[derive(Debug, Clone)]
pub struct VerifyCollection;

#[async_trait]
impl CommandTrait for VerifyCollection {
    fn name(&self) -> Name {
        VERIFY_COLLECTION.into()
    }

    fn inputs(&self) -> Vec<CmdInput> {
        [
            CmdInput {
                name: MINT_ACCOUNT.into(),
                type_bounds: [ValueType::Pubkey, ValueType::Keypair, ValueType::String].to_vec(),
                required: true,
                passthrough: true,
            },
            CmdInput {
                name: FEE_PAYER.into(),
                type_bounds: [ValueType::Keypair, ValueType::String].to_vec(),
                required: true,
                passthrough: true,
            },
            CmdInput {
                name: COLLECTION_AUTHORITY.into(),
                type_bounds: [ValueType::Keypair, ValueType::String].to_vec(),
                required: true,
                passthrough: true,
            },
            CmdInput {
                name: COLLECTION_MINT_ACCOUNT.into(),
                type_bounds: [ValueType::Pubkey, ValueType::Keypair, ValueType::String].to_vec(),
                required: true,
                passthrough: true,
            },
            CmdInput {
                name: COLLECTION_AUTHORITY_IS_DELEGATED.into(),
                type_bounds: [ValueType::Bool].to_vec(),
                required: true,
                passthrough: false,
            },
        ]
        .to_vec()
    }

    fn outputs(&self) -> Vec<CmdOutput> {
        [CmdOutput {
            name: SIGNATURE.into(),
            r#type: ValueType::String,
        }]
        .to_vec()
    }

    async fn run(&self, ctx: Context, inputs: ValueSet) -> Result<ValueSet, CommandError> {
        let Input {
            mint_account,
            fee_payer,
            collection_authority,
            collection_mint_account,
            collection_authority_is_delegated,
        } = value::from_map(inputs)?;

        let (collection_metadata_account, _) =
            mpl_token_metadata::pda::find_metadata_account(&collection_mint_account);

        let (collection_master_edition_account, _) =
            mpl_token_metadata::pda::find_master_edition_account(&collection_mint_account);

        let collection_authority_record = if collection_authority_is_delegated {
            Some(
                mpl_token_metadata::pda::find_collection_authority_account(
                    &mint_account,
                    &collection_authority.pubkey(),
                )
                .0,
            )
        } else {
            None
        };

        let (metadata_account, _) = mpl_token_metadata::pda::find_metadata_account(&mint_account);

        let (minimum_balance_for_rent_exemption, instructions) = self
            .command_verify_collection(
                &ctx.solana_client,
                metadata_account,
                collection_authority.pubkey(),
                fee_payer.pubkey(),
                collection_mint_account,
                collection_metadata_account,
                collection_master_edition_account,
                collection_authority_record,
            )
            .await?;

        let (mut transaction, recent_blockhash) = execute(
            &ctx.solana_client,
            &fee_payer.pubkey(),
            &instructions,
            minimum_balance_for_rent_exemption,
        )
        .await?;

        try_sign_wallet(
            &ctx,
            &mut transaction,
            &[&collection_authority, &fee_payer],
            recent_blockhash,
        )
        .await?;

        let signature = submit_transaction(&ctx.solana_client, transaction).await?;

        Ok(value::to_map(&Output { signature })?)
    }
}

inventory::submit!(CommandDescription::new(VERIFY_COLLECTION, |_| Box::new(
    VerifyCollection
)));

impl VerifyCollection {
    #[allow(clippy::too_many_arguments)]
    async fn command_verify_collection(
        &self,
        rpc_client: &RpcClient,
        metadata: Pubkey,
        collection_authority: Pubkey,
        payer: Pubkey,
        collection_mint: Pubkey,
        collection: Pubkey,
        collection_master_edition_account: Pubkey,
        collection_authority_record: Option<Pubkey>,
    ) -> crate::Result<(u64, Vec<Instruction>)> {

        // FIXME calculcation min balance
        let minimum_balance_for_rent_exemption = rpc_client
            .get_minimum_balance_for_rent_exemption(
                100, // std::mem::size_of::<
                    // mpl_token_metadata::state::VerifyCollection,
                    // >(),
            )
            .await?;

        let instructions = vec![mpl_token_metadata::instruction::verify_collection(
            mpl_token_metadata::id(),
            metadata,
            collection_authority,
            payer,
            collection_mint,
            collection,
            collection_master_edition_account,
            collection_authority_record,
        )];

        Ok((minimum_balance_for_rent_exemption, instructions))
    }
}
