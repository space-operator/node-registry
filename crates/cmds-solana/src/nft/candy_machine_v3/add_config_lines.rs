use crate::{ prelude::*};
use anchor_lang::{InstructionData, ToAccountMetas};
use solana_program::{instruction::Instruction};
use solana_sdk::pubkey::Pubkey;

use mpl_candy_machine_core::instruction::AddConfigLines as MPLAddConfigLines;

use super::ConfigLine;

// Command Name
const ADD_CONFIG_LINES: &str = "add_config_lines";

#[derive(Debug)]
pub struct AddConfigLines;

#[derive(Serialize, Deserialize, Debug)]
pub struct Input {
    #[serde(with = "value::pubkey")]
    pub candy_machine: Pubkey,
    #[serde(with = "value::keypair")]
    pub authority: Keypair,
    #[serde(with = "value::keypair")]
    pub payer: Keypair,
    pub index: u32,
    pub config_lines: Vec<ConfigLine>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    #[serde(with = "value::signature")]
    signature: Signature,
}

#[async_trait]
impl CommandTrait for AddConfigLines {
    fn name(&self) -> Name {
        ADD_CONFIG_LINES.into()
    }

    fn inputs(&self) -> Vec<CmdInput> {
        [
            CmdInput {
                name: "CANDY_MACHINE".into(),
                type_bounds: [ValueType::Pubkey, ValueType::Keypair, ValueType::String].to_vec(),
                required: true,
                passthrough: true,
            },
            CmdInput {
                name: "AUTHORITY".into(),
                type_bounds: [ValueType::Keypair, ValueType::String].to_vec(),
                required: true,
                passthrough: false,
            },
            CmdInput {
                name: "PAYER".into(),
                type_bounds: [ValueType::Keypair, ValueType::String].to_vec(),
                required: true,
                passthrough: false,
            },
            CmdInput {
                name: "INDEX".into(),
                type_bounds: [ValueType::U32].to_vec(),
                required: true,
                passthrough: false,
            },
            CmdInput {
                name: "CONFIG_LINES".into(),
                type_bounds: [ValueType::Free].to_vec(),
                required: true,
                passthrough: false,
            },
        ]
        .to_vec()
    }

    fn outputs(&self) -> Vec<CmdOutput> {
        [CmdOutput {
            name: "SIGNATURE".into(),
            r#type: ValueType::String,
        }]
        .to_vec()
    }

    async fn run(&self, ctx: Context, inputs: ValueSet) -> Result<ValueSet, CommandError> {
        let Input {
            candy_machine,
            authority,
            payer,
            index,
            config_lines,
        } = value::from_map(inputs.clone())?;

        let accounts = mpl_candy_machine_core::accounts::AddConfigLines {
            candy_machine,
            authority: authority.pubkey(),
        }
        .to_account_metas(None);

        let data = MPLAddConfigLines {
            index,
            config_lines: config_lines.into_iter().map(Into::into).collect(),
        }
        .data();

        let instructions = vec![Instruction {
            program_id: mpl_candy_machine_core::id(),
            accounts,
            data,
        }];

        let minimum_balance_for_rent_exemption = ctx
            .solana_client
            .get_minimum_balance_for_rent_exemption(std::mem::size_of::<
                mpl_candy_machine_core::accounts::AddConfigLines,
            >())
            .await?;

        let (mut transaction, recent_blockhash) = execute(
            &ctx.solana_client,
            &payer.pubkey(),
            &instructions,
            minimum_balance_for_rent_exemption,
        )
        .await?;

        try_sign_wallet(
            &ctx,
            &mut transaction,
            &[&payer, &authority],
            recent_blockhash,
        )
        .await?;

        let signature = submit_transaction(&ctx.solana_client, transaction).await?;

        Ok(value::to_map(&Output { signature })?)
    }
}

inventory::submit!(CommandDescription::new(ADD_CONFIG_LINES, |_| {
    Box::new(AddConfigLines)
}));

