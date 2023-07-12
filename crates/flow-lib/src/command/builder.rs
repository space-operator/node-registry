use super::{CommandError, CommandTrait};
use crate::{
    command::InstructionInfo,
    config::node::{Definition, Permissions},
    Context, Name,
};
use futures::future::BoxFuture;
use serde::{de::DeserializeOwned, Serialize};
use std::future::Future;
use thiserror::Error as ThisError;

pub type BuildResult = Result<Box<dyn CommandTrait>, CommandError>;

pub type BuilderCache = once_cell::sync::Lazy<Result<CmdBuilder, BuilderError>>;

#[derive(Clone)]
pub struct CmdBuilder {
    def: Definition,
    signature_name: Option<String>,
}

#[derive(ThisError, Debug, Clone)]
pub enum BuilderError {
    #[error("{0}")]
    Json(String),
    #[error("wrong command name: {0}")]
    WrongName(String),
    #[error("output not found: {0}")]
    OutputNotFound(String),
}

impl From<serde_json::Error> for BuilderError {
    fn from(value: serde_json::Error) -> Self {
        BuilderError::Json(value.to_string())
    }
}

impl CmdBuilder {
    pub fn new(def: &str) -> Result<Self, serde_json::Error> {
        let def = serde_json::from_str(def)?;
        Ok(Self {
            def,
            signature_name: None,
        })
    }

    pub fn check_name(self, name: &str) -> Result<Self, BuilderError> {
        if self.def.data.node_id == name {
            Ok(self)
        } else {
            Err(BuilderError::WrongName(self.def.data.node_id))
        }
    }

    pub fn permissions(mut self, p: Permissions) -> Self {
        self.def.permissions = p;
        self
    }

    pub fn simple_instruction_info(mut self, signature_name: &str) -> Result<Self, BuilderError> {
        if self.def.sources.iter().any(|x| x.name == signature_name) {
            self.signature_name = Some(signature_name.to_owned());
            Ok(self)
        } else {
            Err(BuilderError::OutputNotFound(signature_name.to_owned()))
        }
    }

    pub fn build<T, U, Fut, F>(self, f: F) -> Box<dyn CommandTrait>
    where
        F: Fn(Context, T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<U, CommandError>> + Send + 'static,
        T: DeserializeOwned + 'static,
        U: Serialize,
    {
        struct Command<T, Fut> {
            name: Name,
            inputs: Vec<crate::CmdInputDescription>,
            outputs: Vec<crate::CmdOutputDescription>,
            instruction_info: Option<InstructionInfo>,
            permissions: Permissions,
            run: Box<dyn Fn(Context, T) -> Fut + Send + Sync + 'static>,
        }

        impl<T, U, Fut> CommandTrait for Command<T, Fut>
        where
            Fut: Future<Output = Result<U, CommandError>> + Send + 'static,
            T: DeserializeOwned + 'static,
            U: Serialize,
        {
            fn name(&self) -> Name {
                self.name.clone()
            }

            fn instruction_info(&self) -> Option<InstructionInfo> {
                self.instruction_info.clone()
            }

            fn inputs(&self) -> Vec<crate::CmdInputDescription> {
                self.inputs.clone()
            }

            fn outputs(&self) -> Vec<crate::CmdOutputDescription> {
                self.outputs.clone()
            }

            fn run<'a: 'b, 'b>(
                &'a self,
                ctx: Context,
                params: crate::ValueSet,
            ) -> BoxFuture<'b, Result<crate::ValueSet, CommandError>> {
                match value::from_map(params) {
                    Ok(input) => {
                        let fut = (self.run)(ctx, input);
                        Box::pin(async move { Ok(value::to_map(&fut.await?)?) })
                    }
                    Err(error) => Box::pin(async move { Err(error.into()) }),
                }
            }

            fn permissions(&self) -> Permissions {
                self.permissions.clone()
            }
        }

        let mut cmd = Command {
            name: self.def.data.node_id.clone(),
            run: Box::new(f),
            inputs: self
                .def
                .targets
                .into_iter()
                .map(|x| crate::CmdInputDescription {
                    name: x.name,
                    type_bounds: x.type_bounds,
                    required: x.required,
                    passthrough: x.passthrough,
                })
                .collect(),
            outputs: self
                .def
                .sources
                .into_iter()
                .map(|x| crate::CmdOutputDescription {
                    name: x.name,
                    r#type: x.r#type,
                })
                .collect(),
            instruction_info: None,
            permissions: self.def.permissions,
        };

        if let Some(name) = self.signature_name {
            cmd.instruction_info = Some(InstructionInfo::simple(&cmd, &name))
        }

        Box::new(cmd)
    }
}
