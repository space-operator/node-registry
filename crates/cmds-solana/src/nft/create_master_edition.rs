use crate::{prelude::*, proxy_authority::utils::find_proxy_authority_address};
use anchor_lang::InstructionData;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    system_program,
};
use space_wrapper::instruction::ProxyCreateMasterEditionV3;

#[derive(Debug, Clone)]
pub struct CreateMasterEdition;

pub fn create_proxy_create_master_edition_instruction(
    authority: &Pubkey,
    proxy_authority: &Pubkey,
    edition: &Pubkey,
    mint: &Pubkey,
    mint_authority: &Pubkey,
    metadata: &Pubkey,
    token_metadata_program: &Pubkey,
    token_program: &Pubkey,
    max_supply: u64,
) -> Instruction {
    Instruction {
        program_id: space_wrapper::ID,
        accounts: [
            AccountMeta::new_readonly(*authority, true),
            AccountMeta::new(*proxy_authority, false),
            AccountMeta::new(*edition, false),
            AccountMeta::new(*mint, false),
            AccountMeta::new_readonly(*mint_authority, true),
            AccountMeta::new(*metadata, false),
            AccountMeta::new_readonly(*token_metadata_program, false),
            AccountMeta::new_readonly(*token_program, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ]
        .to_vec(),
        data: ProxyCreateMasterEditionV3 {
            max_supply: Some(max_supply),
        }
        .data(),
    }
}

impl CreateMasterEdition {
    #[allow(clippy::too_many_arguments)]
    async fn command_create_master_edition(
        &self,
        rpc_client: &RpcClient,
        metadata_pubkey: Pubkey,
        master_edition_pubkey: Pubkey,
        mint: Pubkey,
        mint_authority: Pubkey,
        payer: Pubkey,
        update_authority: Pubkey,
        max_supply: u64,
    ) -> crate::Result<(u64, Vec<Instruction>)> {
        let minimum_balance_for_rent_exemption = rpc_client
            .get_minimum_balance_for_rent_exemption(std::mem::size_of::<
                mpl_token_metadata::state::MasterEditionV2,
            >())
            .await?;

        let instruction = mpl_token_metadata::instruction::create_master_edition_v3(
            mpl_token_metadata::id(),
            master_edition_pubkey,
            mint,
            update_authority,
            mint_authority,
            metadata_pubkey,
            payer,
            Some(max_supply),
        );

        Ok((minimum_balance_for_rent_exemption, [instruction].to_vec()))
    }

    #[allow(clippy::too_many_arguments)]
    async fn command_proxy_create_metadata_accounts(
        &self,
        rpc_client: &RpcClient,
        metadata_pubkey: Pubkey,
        master_edition_pubkey: Pubkey,
        mint: Pubkey,
        mint_authority: Pubkey,
        payer: Pubkey,
        proxy_authority: Pubkey,
        max_supply: u64,
    ) -> crate::Result<(u64, Vec<Instruction>)> {
        let minimum_balance_for_rent_exemption = rpc_client
            .get_minimum_balance_for_rent_exemption(std::mem::size_of::<
                mpl_token_metadata::state::MasterEditionV2,
            >())
            .await?;

        let instruction = create_proxy_create_master_edition_instruction(
            &payer,
            &proxy_authority,
            &master_edition_pubkey,
            &mint,
            &mint_authority,
            &metadata_pubkey,
            &mpl_token_metadata::id(),
            &spl_token::id(),
            max_supply,
        );

        Ok((minimum_balance_for_rent_exemption, [instruction].to_vec()))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Input {
    #[serde(flatten)]
    update_authority: UpdateAuthority,
    #[serde(with = "value::pubkey")]
    pub mint_account: Pubkey,
    #[serde(with = "value::pubkey")]
    pub mint_authority: Pubkey,
    #[serde(with = "value::keypair")]
    pub fee_payer: Keypair,
    pub max_supply: u64,
    #[serde(default = "value::default::bool_true")]
    pub submit: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum UpdateAuthority {
    NoProxy {
        #[serde(with = "value::keypair")]
        update_authority: Keypair,
    },
    Proxy {
        #[serde(with = "value::pubkey")]
        proxy_as_update_authority: Pubkey,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    #[serde(default, with = "value::signature::opt")]
    pub signature: Option<Signature>,
}

const CREATE_MASTER_EDITION: &str = "create_master_edition";

// Inputs
const PROXY_AS_UPDATE_AUTHORITY: &str = "proxy_as_update_authority";
const UPDATE_AUTHORITY: &str = "update_authority";
const MINT_ACCOUNT: &str = "mint_account";
const MINT_AUTHORITY: &str = "mint_authority";
const FEE_PAYER: &str = "fee_payer";
const MAX_SUPPLY: &str = "max_supply";
const SUBMIT: &str = "submit";

// Outputs
const SIGNATURE: &str = "signature";
const METADATA_ACCOUNT: &str = "metadata_account";
const MASTER_EDITION_ACCOUNT: &str = "master_edition_account";

#[async_trait]
impl CommandTrait for CreateMasterEdition {
    fn instruction_info(&self) -> Option<InstructionInfo> {
        Some(InstructionInfo::simple(self, SIGNATURE))
    }

    fn name(&self) -> Name {
        CREATE_MASTER_EDITION.into()
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
                name: MINT_AUTHORITY.into(),
                type_bounds: [ValueType::Pubkey, ValueType::Keypair, ValueType::String].to_vec(),
                required: true,
                passthrough: false,
            },
            CmdInput {
                name: FEE_PAYER.into(),
                type_bounds: [ValueType::Keypair, ValueType::String].to_vec(),
                required: true,
                passthrough: true,
            },
            CmdInput {
                name: UPDATE_AUTHORITY.into(),
                type_bounds: [ValueType::Keypair, ValueType::String].to_vec(),
                required: false,
                passthrough: false,
            },
            CmdInput {
                name: MAX_SUPPLY.into(),
                type_bounds: [ValueType::U64].to_vec(),
                required: true,
                passthrough: false,
            },
            CmdInput {
                name: SUBMIT.into(),
                type_bounds: [ValueType::Bool].to_vec(),
                required: false,
                passthrough: false,
            },
            CmdInput {
                name: PROXY_AS_UPDATE_AUTHORITY.into(),
                type_bounds: [ValueType::Pubkey].to_vec(),
                required: false,
                passthrough: true,
            },
        ]
        .to_vec()
    }
    fn outputs(&self) -> Vec<CmdOutput> {
        [
            CmdOutput {
                name: SIGNATURE.into(),
                r#type: ValueType::String,
            },
            CmdOutput {
                name: METADATA_ACCOUNT.into(),
                r#type: ValueType::Pubkey,
            },
            CmdOutput {
                name: MASTER_EDITION_ACCOUNT.into(),
                r#type: ValueType::Pubkey,
            },
        ]
        .to_vec()
    }

    async fn run(&self, mut ctx: Context, inputs: ValueSet) -> Result<ValueSet, CommandError> {
        let inputs: Input = value::from_map(inputs)?;

        let (metadata_account, _) =
            mpl_token_metadata::pda::find_metadata_account(&inputs.mint_account);

        let (master_edition_account, _) =
            mpl_token_metadata::pda::find_master_edition_account(&inputs.mint_account);

        let (minimum_balance_for_rent_exemption, instructions, signers) =
            match &inputs.update_authority {
                UpdateAuthority::Proxy { .. } => {
                    let proxy_authority = find_proxy_authority_address(&inputs.fee_payer.pubkey());

                    let (minimum_balance_for_rent_exemption, instructions) = self
                        .command_proxy_create_metadata_accounts(
                            &ctx.solana_client,
                            metadata_account,
                            master_edition_account,
                            inputs.mint_account,
                            inputs.mint_authority,
                            inputs.fee_payer.pubkey(),
                            proxy_authority,
                            inputs.max_supply,
                        )
                        .await?;

                    (
                        minimum_balance_for_rent_exemption,
                        instructions,
                        vec![inputs.fee_payer.clone_keypair()],
                    )
                }
                UpdateAuthority::NoProxy { update_authority } => {
                    let (minimum_balance_for_rent_exemption, instructions) = self
                        .command_create_master_edition(
                            &ctx.solana_client,
                            metadata_account,
                            master_edition_account,
                            inputs.mint_account,
                            inputs.mint_authority,
                            inputs.fee_payer.pubkey(),
                            update_authority.pubkey(),
                            inputs.max_supply,
                        )
                        .await?;

                    (
                        minimum_balance_for_rent_exemption,
                        instructions,
                        vec![
                            inputs.fee_payer.clone_keypair(),
                            update_authority.clone_keypair(),
                        ],
                    )
                }
            };

        let instructions = if inputs.submit {
            Instructions {
                fee_payer: inputs.fee_payer.pubkey(),
                signers,
                minimum_balance_for_rent_exemption,
                instructions,
            }
        } else {
            Instructions::default()
        };

        let signature = ctx
            .execute(
                instructions,
                value::map! {
                    METADATA_ACCOUNT => metadata_account,
                    MASTER_EDITION_ACCOUNT => master_edition_account,
                },
            )
            .await?
            .signature;

        Ok(value::to_map(&Output { signature })?)
    }
}

inventory::submit!(CommandDescription::new(
    CREATE_MASTER_EDITION,
    |_| Box::new(CreateMasterEdition)
));
