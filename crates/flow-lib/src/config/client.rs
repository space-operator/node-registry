//! Parse JS front-end flow config into back-end flow config

use crate::{CommandType, FlowId, SolanaNet, ValueType};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use uuid::Uuid;

struct SeqVisitor<T> {
    marker: std::marker::PhantomData<T>,
    dbg_info: &'static str,
}

impl<'a, T> serde::de::Visitor<'a> for SeqVisitor<T>
where
    T: Sized + Deserialize<'a>,
{
    type Value = Vec<T>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::write(formatter, format_args!("array"))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'a>,
    {
        let mut vec = Vec::with_capacity(seq.size_hint().unwrap_or(0));
        loop {
            match seq.next_element() {
                Ok(Some(value)) => vec.push(value),
                Ok(None) => break,
                Err(error) => {
                    log::warn!("ignoring {}: {}", self.dbg_info, error);
                    continue;
                }
            }
        }
        Ok(vec)
    }
}

fn ignore_error<'de, D, T>(de: D, dbg_info: &'static str) -> Result<Vec<T>, D::Error>
where
    D: serde::de::Deserializer<'de>,
    T: Sized + Deserialize<'de>,
{
    de.deserialize_seq(SeqVisitor {
        marker: std::marker::PhantomData,
        dbg_info,
    })
}

fn ignore_error_node<'de, D>(de: D) -> Result<Vec<Node>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    ignore_error(de, "node")
}

fn ignore_error_edge<'de, D>(de: D) -> Result<Vec<Edge>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    ignore_error(de, "edge")
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientConfig {
    pub id: FlowId,
    pub network: SolanaNet,
    #[serde(deserialize_with = "ignore_error_node")]
    pub nodes: Vec<Node>,
    #[serde(deserialize_with = "ignore_error_edge")]
    pub edges: Vec<Edge>,
    #[serde(default)]
    pub environment: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Node {
    pub id: Uuid,
    pub data: NodeData,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeData {
    pub r#type: CommandType,
    pub node_id: String,
    pub sources: Vec<Source>,
    pub targets: Vec<Target>,
    pub targets_form: TargetsForm,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Source {
    pub id: Uuid,
    pub name: String,
    pub r#type: ValueType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Target {
    pub id: Uuid,
    pub name: String,
    pub type_bounds: Vec<ValueType>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetsForm {
    pub form_data: JsonValue,
    #[serde(default)]
    pub extra: Extra,

    #[serde(skip)]
    pub wasm_bytes: Option<bytes::Bytes>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Extra {
    // for WASM node
    pub supabase_id: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceHandle {
    pub id: Uuid,
    pub is_passthough: bool,
}

impl Serialize for SourceHandle {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if self.is_passthough {
            format!("passthrough-{}", self.id).serialize(s)
        } else {
            self.id.serialize(s)
        }
    }
}

impl<'de> Deserialize<'de> for SourceHandle {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const PREFIX: &str = "passthrough-";
        let s = String::deserialize(d)?;
        let (is_passthough, uuid_str) = if s.starts_with(PREFIX) {
            (true, &s.as_str()[PREFIX.len()..])
        } else {
            (false, s.as_str())
        };

        Ok(Self {
            is_passthough,
            id: uuid_str.parse().map_err(serde::de::Error::custom)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Edge {
    pub source: Uuid,
    #[serde(rename = "sourceHandle")]
    pub source_handle: SourceHandle,
    pub target: Uuid,
    #[serde(rename = "targetHandle")]
    pub target_handle: Uuid,
}
