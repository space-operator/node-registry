use crate::{
    prelude::*,
    utils::{execute, submit_transaction},
};
use solana_program::instruction::Instruction;

#[derive(Debug, Clone)]
pub struct ApproveCollectionAuthority;

impl ApproveCollectionAuthority {
    #[allow(clippy::too_many_arguments)]
    pub async fn command_approve_collection_authority(
        &self,
        rpc_client: &RpcClient,
        collection_authority_record: Pubkey,
        new_collection_authority: Pubkey,
        update_authority: Pubkey,
        payer: Pubkey,
        metadata: Pubkey,
        mint: Pubkey,
    ) -> crate::Result<(u64, Vec<Instruction>)> {
        let minimum_balance_for_rent_exemption = rpc_client
            .get_minimum_balance_for_rent_exemption(std::mem::size_of::<
                mpl_token_metadata::state::CollectionAuthorityRecord,
            >())
            .await?;

        let instructions = vec![
            mpl_token_metadata::instruction::approve_collection_authority(
                mpl_token_metadata::id(),
                collection_authority_record,
                new_collection_authority,
                update_authority,
                payer,
                metadata,
                mint,
            ),
        ];

        Ok((minimum_balance_for_rent_exemption, instructions))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Input {
    #[serde(with = "value::pubkey")]
    pub new_collection_authority: Pubkey,
    #[serde(with = "value::keypair")]
    pub update_authority: Keypair,
    #[serde(with = "value::keypair")]
    pub fee_payer: Keypair,
    #[serde(with = "value::pubkey")]
    pub mint_account: Pubkey,
    #[serde(default = "value::default::bool_true")]
    pub submit: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    #[serde(default, with = "value::signature::opt")]
    pub signature: Option<Signature>,
}

const APPROVE_COLLECTION_AUTHORITY: &str = "approve_collection_authority";

// Inputs
const NEW_COLLECTION_AUTHORITY: &str = "new_collection_authority";
const UPDATE_AUTHORITY: &str = "update_authority";
const FEE_PAYER: &str = "fee_payer";
const MINT_ACCOUNT: &str = "mint_account";
const SUBMIT: &str = "submit";

// Output
const SIGNATURE: &str = "signature";

#[async_trait]
impl CommandTrait for ApproveCollectionAuthority {
    fn name(&self) -> Name {
        APPROVE_COLLECTION_AUTHORITY.into()
    }

    fn inputs(&self) -> Vec<CmdInput> {
        [
            CmdInput {
                name: NEW_COLLECTION_AUTHORITY.into(),
                type_bounds: [ValueType::Pubkey].to_vec(),
                required: true,
                passthrough: false,
            },
            CmdInput {
                name: UPDATE_AUTHORITY.into(),
                type_bounds: [ValueType::Keypair].to_vec(),
                required: true,
                passthrough: true,
            },
            CmdInput {
                name: FEE_PAYER.into(),
                type_bounds: [ValueType::Keypair].to_vec(),
                required: true,
                passthrough: true,
            },
            CmdInput {
                name: MINT_ACCOUNT.into(),
                type_bounds: [ValueType::Pubkey].to_vec(),
                required: true,
                passthrough: true,
            },
            CmdInput {
                name: SUBMIT.into(),
                type_bounds: [ValueType::Bool].to_vec(),
                required: false,
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
            new_collection_authority,
            update_authority,
            fee_payer,
            mint_account,
            submit,
        } = value::from_map(inputs)?;

        let program_id = mpl_token_metadata::id();

        let metadata_seeds = &[
            mpl_token_metadata::state::PREFIX.as_bytes(),
            program_id.as_ref(),
            mint_account.as_ref(),
        ];

        let (metadata_pubkey, _) = Pubkey::find_program_address(metadata_seeds, &program_id);

        let (collection_authority_record, _) =
            mpl_token_metadata::pda::find_collection_authority_account(
                &mint_account,
                &new_collection_authority,
            );

        let (minimum_balance_for_rent_exemption, instructions) = self
            .command_approve_collection_authority(
                &ctx.solana_client,
                collection_authority_record,
                new_collection_authority,
                update_authority.pubkey(),
                fee_payer.pubkey(),
                metadata_pubkey,
                mint_account,
            )
            .await?;

        let fee_payer_pubkey = fee_payer.pubkey();

        let (mut transaction, recent_blockhash) = execute(
            &ctx.solana_client,
            &fee_payer_pubkey,
            &instructions,
            minimum_balance_for_rent_exemption,
        )
        .await?;
        try_sign_wallet(
            &ctx,
            &mut transaction,
            &[&update_authority, &fee_payer],
            recent_blockhash,
        )
        .await?;

        let signature = if submit {
            Some(submit_transaction(&ctx.solana_client, transaction).await?)
        } else {
            None
        };

        Ok(value::to_map(&Output { signature })?)
    }
}

inventory::submit!(CommandDescription::new(
    APPROVE_COLLECTION_AUTHORITY,
    |_| Box::new(ApproveCollectionAuthority)
));
