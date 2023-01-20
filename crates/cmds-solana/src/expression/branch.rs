use crate::{prelude::*, Error};
use async_trait::async_trait;
use flow_lib::{command::CommandTrait, Context, Name, ValueSet};
use indexmap::indexmap;
use serde_json::Value as JsonValue;
use value::{Value, ValueType};

#[derive(Debug)]
pub struct BranchCommand;

impl BranchCommand {
    // Counts the number of slots until the first unused.
    fn count_unique_slots(expression: &str) -> usize {
        for i in 0..usize::MAX {
            if !expression.contains(&format!("${{{}}}", i)) {
                return i;
            }
        }
        0
    }
}
 
pub const BRANCH_CMD: &str = "branch";

// Inputs
const EXPRESSION: &str = "expression";
const VALUES: &str = "values";

// Outputs
const TRUE_BRANCH: &str = "__true_branch";
const FALSE_BRANCH: &str = "__false_branch";

#[async_trait]
impl CommandTrait for BranchCommand {
    fn name(&self) -> Name {
        BRANCH_CMD.into()
    }

    fn inputs(&self) -> Vec<CmdInput> {
        [
            CmdInput {
                name: EXPRESSION.into(),
                type_bounds: [ValueType::String].to_vec(),
                required: true,
                passthrough: false,
            },
            CmdInput {
                name: VALUES.into(),
                type_bounds: [ValueType::Free].to_vec(),
                required: true,
                passthrough: false,
            },
        ]
        .to_vec()
    }

    fn outputs(&self) -> Vec<CmdOutput> {
        [
            CmdOutput {
                name: TRUE_BRANCH.into(),
                r#type: ValueType::Free,
            },
            CmdOutput {
                name: FALSE_BRANCH.into(),
                r#type: ValueType::Free,
            },
        ]
        .to_vec()
    }

    async fn run(&self, _ctx: Context, mut inputs: ValueSet) -> Result<ValueSet, CommandError> {
        let mut expression = if let Value::String(expression) = inputs
            .remove(EXPRESSION)
            .ok_or_else(|| crate::Error::ValueNotFound(EXPRESSION.into()))?
        {
            expression
        } else {
            panic!("Wrong expression value!");
        };
        let values = match inputs
            .remove(VALUES)
            .ok_or_else(|| crate::Error::ValueNotFound(EXPRESSION.into()))?
        {
            Value::Array(values) => values,
            _ => {
                return Err(Error::RhaiExecutionError(
                    "Values passed aren't JSON array".to_string(),
                ))
            }
        };
        let slots = Self::count_unique_slots(&expression);

        if values.len() != slots {
            return Err(Error::RhaiExecutionError(
                "Number of values and expression slots don't match".to_string(),
            ));
        }
        for (index, val) in values.iter().enumerate() {
            match val {
                Value::String(s) => {
                    expression = expression
                        .as_str()
                        .replace(&format!("${{{}}}", index), &format!("\"{}\"", s));
                }
                Value::U32(n) => {
                    expression = expression.replace(&format!("${{{}}}", index), &format!("{}", n));
                }
                Value::I32(n) => {
                    expression = expression.replace(&format!("${{{}}}", index), &format!("{}", n));
                }
                Value::I8(n) => {
                    expression = expression.replace(&format!("${{{}}}", index), &format!("{}", n));
                }
                Value::I16(n) => {
                    expression = expression.replace(&format!("${{{}}}", index), &format!("{}", n));
                }
                Value::U8(n) => {
                    expression = expression.replace(&format!("${{{}}}", index), &format!("{}", n));
                }
                Value::U16(n) => {
                    expression = expression.replace(&format!("${{{}}}", index), &format!("{}", n));
                }
                Value::U128(n) => {
                    expression = expression.replace(&format!("${{{}}}", index), &format!("{}", n));
                }
                Value::F32(n) => {
                    expression = expression.replace(&format!("${{{}}}", index), &format!("{}", n));
                }
                Value::F64(n) => {
                    expression = expression.replace(&format!("${{{}}}", index), &format!("{}", n));
                }
                _ => {
                    panic!("Currently not supported!");
                }
            }
        }

        let engine = rhai::Engine::new();

        let exp = engine
            .eval::<Vec<rhai::Dynamic>>(&expression)
            .map_err(|e| Error::RhaiExecutionError(e.to_string()))?;

        let exp = exp
            .into_iter()
            .map(|a| match a.type_name() {
                "i64" => {
                    let v: Option<i64> = a.try_cast();
                    if let Some(v) = v {
                        JsonValue::from(v)
                    } else {
                        JsonValue::Null
                    }
                }
                "f64" => {
                    let v: Option<f64> = a.try_cast();
                    if let Some(v) = v {
                        JsonValue::from(v)
                    } else {
                        JsonValue::Null
                    }
                }
                "string" => {
                    let v: Option<String> = a.try_cast();
                    if let Some(v) = v {
                        JsonValue::from(v)
                    } else {
                        JsonValue::Null
                    }
                }
                "bool" => {
                    let v: Option<bool> = a.try_cast();
                    if let Some(v) = v {
                        JsonValue::from(v)
                    } else {
                        JsonValue::Null
                    }
                }
                _ => {
                    panic!("Currently not supported");
                }
            })
            .collect::<Vec<JsonValue>>();

        let branch = if let Some(JsonValue::Bool(true)) = exp.get(0) {
            TRUE_BRANCH
        } else {
            FALSE_BRANCH
        };
        // let branch = if exp { TRUE_BRANCH } else { FALSE_BRANCH };
        Ok(indexmap! {
           branch.into() => Value::from_json_value(exp.get(1).cloned().unwrap_or(JsonValue::Null)),
        })
    }
}

inventory::submit!(CommandDescription::new(BRANCH_CMD, |_| {
    Box::new(BranchCommand {})
}));

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use flow_lib::{command::CommandTrait, Context, ValueSet};
    use indexmap::indexmap;
    use value::Value;

    use super::{BranchCommand, EXPRESSION, VALUES};

    #[tokio::test]
    async fn test_simple_command() {
        let cmd = BranchCommand {};
        let ctx = Context::default();

        // Compare integers
        let inputs: ValueSet = indexmap! {
            EXPRESSION.into() => Value::String("[${0} > ${1}, ${0}]".to_string()),
            VALUES.into() => Value::Array(vec![Value::U32(1), Value::U32(2)]),
        };

        let outputs = cmd.run(ctx.clone(), inputs).await;
        assert!(outputs.is_ok());
    }

    #[tokio::test]
    async fn test_complex_command() {
        let cmd = BranchCommand {};
        let ctx = Context::default();

        let expression = r#"
            let comparison = (${0} * ${1} / ${0} * ${2}) - ${3};
            if comparison > 0 {
                [true, "The comparison is positive"]
            }else{
                [false, "The comparison is negative"]
            }
        "#;

        // Compare integers
        let inputs: ValueSet = indexmap! {
            EXPRESSION.into() => Value::String(expression.to_string()),
            VALUES.into() => Value::Array(vec![Value::U32(1), Value::U32(2), Value::U32(3), Value::U32(5)]),
        };

        let outputs = cmd.run(ctx.clone(), inputs).await;
        dbg!(&outputs);
        assert!(outputs.is_ok());
        let outputs = outputs.unwrap();
        let o = outputs.get("__true_branch");
        assert!(o.is_some());
        let o = o.unwrap();
        assert_eq!(o, &Value::String("The comparison is positive".into()));
    }

    #[tokio::test]
    async fn test_simple_comparison() {
        let cmd = BranchCommand {};
        let ctx = Context::default();

        // Compare integers
        let inputs: ValueSet = indexmap! {
            EXPRESSION.into() => Value::String("[${0} > ${1}]".to_string()),
            VALUES.into() => Value::Array(vec![Value::U32(1), Value::U32(2)]),
        };

        let outputs = cmd.run(ctx.clone(), inputs).await;
        assert!(outputs.is_ok());

        // Compare mixed types
        let inputs: ValueSet = indexmap! {
            EXPRESSION.into() => Value::String("[${0} < ${1}]".to_string()),
            VALUES.into() => Value::Array(vec![Value::String("1".to_string()), Value::U32(2)]),
        };

        let outputs = cmd.run(ctx.clone(), inputs).await;
        assert!(outputs.is_ok());

        // Compare strings
        let inputs: ValueSet = indexmap! {
            EXPRESSION.into() => Value::String("[${0} == ${1}]".to_string()),
            VALUES.into() => Value::Array(vec![Value::String("1".to_string()), Value::String("2".to_string())]),
        };

        let outputs = cmd.run(ctx.clone(), inputs).await;
        assert!(outputs.is_ok());
    }

    #[tokio::test]
    async fn text_missing_arguments() {
        let cmd = BranchCommand {};
        let ctx = Context::default();

        // More values than expression slots
        let inputs: ValueSet = indexmap! {
            EXPRESSION.into() => Value::String("[${0} > ${1}]".to_string()),
            VALUES.into() => Value::Array(vec![Value::U32(1), Value::U32(2), Value::U32(3)]),
        };

        let outputs = cmd.run(ctx.clone(), inputs).await;
        dbg!(&outputs);

        // More expression slots than values
        let inputs: ValueSet = indexmap! {
            EXPRESSION.into() => Value::String("[${0} > ${1} && ${1} > ${2}]".to_string()),
            VALUES.into() => Value::Array(vec![Value::U32(1), Value::U32(2)]),
        };

        let outputs = cmd.run(ctx.clone(), inputs).await;
        assert!(outputs.is_err());

        // No slots
        let inputs: ValueSet = indexmap! {
            EXPRESSION.into() => Value::String("[1 > 2]".to_string()),
            VALUES.into() => Value::Array(vec![Value::U32(1), Value::U32(2)]),
        };

        let outputs = cmd.run(ctx.clone(), inputs).await;
        assert!(outputs.is_err());

        // No values
        let inputs: ValueSet = indexmap! {
            EXPRESSION.into() => Value::String("[${0} == ${1}]".to_string()),
            VALUES.into() => Value::Array(vec![]),
        };

        let outputs = cmd.run(ctx.clone(), inputs).await;
        assert!(outputs.is_err());

        // No inputs
        let inputs: ValueSet = indexmap! {
            EXPRESSION.into() => Value::String("[1 > 2]".to_string()),
            VALUES.into() => Value::Array(vec![]),
        };

        let outputs = cmd.run(ctx.clone(), inputs).await;
        assert!(outputs.is_ok());
    }
}
