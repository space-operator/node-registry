use crate::{prelude::*, proxy_authority::utils::find_proxy_authority_address};
use anchor_lang::InstructionData;
use mpl_token_metadata::{
    accounts::{MasterEdition, Metadata},
    instructions::{CreateMasterEditionV3InstructionArgs, CreateV1Builder},
};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    stake::state::Meta,
    system_program,
};

// Command Name
const NAME: &str = "create_master_edition";

const DEFINITION: &str =
    include_str!("../../../../node-definitions/solana/NFT/create_master_edition.json");

fn build() -> Result<Box<dyn CommandTrait>, CommandError> {
    use once_cell::sync::Lazy;
    static CACHE: Lazy<Result<CmdBuilder, BuilderError>> = Lazy::new(|| {
        CmdBuilder::new(DEFINITION)?
            .check_name(NAME)?
            .simple_instruction_info("signature")
    });
    Ok(CACHE.clone()?.build(run))
}

inventory::submit!(CommandDescription::new(NAME, |_| { build() }));

#[derive(Serialize, Deserialize, Debug)]
pub struct Input {
    #[serde(with = "value::keypair")]
    update_authority: Keypair,
    #[serde(with = "value::pubkey")]
    pub mint_account: Pubkey,
    #[serde(with = "value::pubkey")]
    pub mint_authority: Pubkey,
    #[serde(with = "value::keypair")]
    pub fee_payer: Keypair,
    pub max_supply: Option<u64>,
    #[serde(default = "value::default::bool_true")]
    pub submit: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    #[serde(default, with = "value::signature::opt")]
    pub signature: Option<Signature>,
}

async fn run(mut ctx: Context, input: Input) -> Result<Output, CommandError> {
    let (metadata_account, _) = Metadata::find_pda(&input.mint_account);

    let (master_edition_account, _) = MasterEdition::find_pda(&input.mint_account);

    let minimum_balance_for_rent_exemption = ctx
        .solana_client
        .get_minimum_balance_for_rent_exemption(std::mem::size_of::<
            mpl_token_metadata::accounts::MasterEdition,
        >())
        .await?;

    let create_ix = CreateV1Builder::new()
        .metadata(metadata_account)
        .master_edition(Some(master_edition_account))
        .mint(input.mint_account, true)
        .authority(input.mint_authority)
        .payer(input.fee_payer.pubkey())
        .update_authority(input.update_authority.pubkey(), true)
        .is_mutable(true)
        .primary_sale_happened(false)
        .name(String::from("NonFungible"))
        .symbol(String::from("NFT"))
        .uri(String::from("http://my.nft"))
        .seller_fee_basis_points(500)
        .token_standard(TokenStandard::NonFungible)
        .print_supply(PrintSupply::Zero)
        .instruction();

    let ins = Instructions {
        fee_payer: input.payer.pubkey(),
        signers: [
            input.fee_payer.clone_keypair(),
            input.update_authority.clone_keypair(),
        ]
        .into(),
        instructions: [create_ix].into(),
        minimum_balance_for_rent_exemption,
    };

    let ins = input.submit.then_some(ins).unwrap_or_default();

    let signature = ctx
        .execute(
            ins,
            value::map! {
                "metadata_account" => metadata_account,
                "master_edition_account" => master_edition_account,
            },
        )
        .await?
        .signature;

    Ok(Output { signature })
}
