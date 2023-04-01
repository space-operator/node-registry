use crate::{
    config::{client::NodeData, CmdInputDescription, CmdOutputDescription, Name, ValueSet},
    context::Context,
    ValueType,
};
use std::borrow::Cow;
use value::Value;

pub type CommandError = anyhow::Error;

#[async_trait::async_trait]
pub trait CommandTrait: Send + Sync + 'static {
    fn name(&self) -> Name;

    fn inputs(&self) -> Vec<CmdInputDescription>;

    fn outputs(&self) -> Vec<CmdOutputDescription>;

    async fn run(&self, ctx: Context, params: ValueSet) -> Result<ValueSet, CommandError>;

    fn read_form_data(&self, data: serde_json::Value) -> ValueSet {
        let mut result = ValueSet::new();
        for i in self.inputs() {
            if let Some(json) = data.get(&i.name) {
                let value = Value::from(json.clone());
                result.insert(i.name.clone(), value);
            }
        }
        result
    }

    fn passthrough_outputs(&self, inputs: &ValueSet) -> ValueSet {
        let mut res = ValueSet::new();
        for i in self.inputs() {
            if i.passthrough {
                if let Some(value) = inputs.get(&i.name) {
                    if !i.required && matches!(value, Value::Null) {
                        continue;
                    }

                    let value = match i.type_bounds.first() {
                        Some(ValueType::Pubkey) => {
                            value::pubkey::deserialize(value.clone()).map(Into::into)
                        }
                        Some(ValueType::Keypair) => {
                            value::keypair::deserialize(value.clone()).map(Into::into)
                        }
                        Some(ValueType::Signature) => {
                            value::signature::deserialize(value.clone()).map(Into::into)
                        }
                        Some(ValueType::Decimal) => {
                            value::decimal::deserialize(value.clone()).map(Into::into)
                        }
                        _ => Ok(value.clone()),
                    }
                    .unwrap_or_else(|error| {
                        tracing::warn!("error reading passthrough: {}", error);
                        value.clone()
                    });
                    res.insert(i.name, value);
                }
            }
        }
        res
    }

    fn instruction_info(&self) -> Option<InstructionInfo> {
        None
    }
}

#[derive(Debug, Clone)]
pub struct InstructionInfo {
    pub before: Vec<Name>,
    pub signature: Name,
    pub after: Vec<Name>,
}

impl InstructionInfo {
    /// before: All passthroughs and outputs, except for `signature`.
    /// after: empty.
    pub fn simple<C: CommandTrait>(cmd: &C, signature: &str) -> Self {
        let before = cmd
            .inputs()
            .into_iter()
            .filter(|i| i.passthrough)
            .map(|i| i.name)
            .chain(
                cmd.outputs()
                    .into_iter()
                    .filter(|o| o.name != signature)
                    .map(|o| o.name),
            )
            .collect();
        Self {
            before,
            after: Vec::new(),
            signature: signature.into(),
        }
    }
}

#[derive(Clone)]
pub struct CommandDescription {
    pub name: Cow<'static, str>,
    pub fn_new: fn(&NodeData) -> Box<dyn CommandTrait>,
}

impl CommandDescription {
    pub const fn new(name: &'static str, fn_new: fn(&NodeData) -> Box<dyn CommandTrait>) -> Self {
        Self {
            name: Cow::Borrowed(name),
            fn_new,
        }
    }
}

inventory::collect!(CommandDescription);
