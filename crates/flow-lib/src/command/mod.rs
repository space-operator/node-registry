use crate::{
    config::{client::NodeData, CmdInputDescription, CmdOutputDescription, Name, ValueSet},
    context::Context,
};
use std::borrow::Cow;
// use value::Value;

pub type CommandError = anyhow::Error;

#[async_trait::async_trait]
pub trait CommandTrait: Send + Sync + 'static {
    fn name(&self) -> Name;

    fn inputs(&self) -> Vec<CmdInputDescription>;

    fn outputs(&self) -> Vec<CmdOutputDescription>;

    async fn run(&self, ctx: Context, params: ValueSet) -> Result<ValueSet, CommandError>;

    fn read_form_data(&self, _: serde_json::Value) -> ValueSet {
        ValueSet::new()
        /*
         * TODO
        let mut result = ValueSet::new();
        for i in self.inputs() {
            if let Some(json) = data.get(&i.name) {
                result.insert(i.name.clone(), Value::from_json_value(json.clone()));
            }
        }
        result
        */
    }

    fn passthrough_outputs(&self, inputs: &ValueSet) -> ValueSet {
        let mut res = ValueSet::new();
        for i in self.inputs() {
            if i.passthrough {
                if let Some(value) = inputs.get(&i.name) {
                    res.insert(i.name, value.clone());
                }
            }
        }
        res
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
