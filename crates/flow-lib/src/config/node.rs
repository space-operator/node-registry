use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub r#type: super::CommandType,
    pub data: Data,
    pub sources: Vec<Source>,
    pub targets: Vec<Target>,
    #[serde(rename = "targets_form.json_schema")]
    pub json_schema: serde_json::Map<String, JsonValue>,
    #[serde(rename = "targets_form.ui_schema")]
    pub ui_schema: serde_json::Map<String, JsonValue>,
    #[serde(default)]
    pub permissions: Permissions,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Permissions {
    #[serde(default)]
    pub user_tokens: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Data {
    pub node_id: String,
    pub version: String,
    pub display_name: String,
    pub description: String,
    pub width: u32,
    pub height: u32,
    #[serde(rename = "backgroundColor")]
    pub background_color: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub name: String,
    pub r#type: super::ValueType,
    #[serde(rename = "defaultValue")]
    pub default_value: JsonValue,
    pub tooltip: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Target {
    pub name: String,
    pub type_bounds: Vec<super::ValueType>,
    pub required: bool,
    pub passthrough: bool,
    #[serde(rename = "defaultValue")]
    pub default_value: JsonValue,
    pub tooltip: String,
}

#[cfg(test)]
mod tests {
    use walkdir::WalkDir;

    use super::*;

    #[test]
    fn test_parse_all() {
        let root = std::env::var("CARGO_MANIFEST_DIR").unwrap() + "/../../node-definitions";
        for e in WalkDir::new(&root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path().is_file() && matches!(e.path().extension(), Some(ex) if ex == "json")
            })
        {
            println!(
                "reading: {}",
                e.path()
                    .to_string_lossy()
                    .strip_prefix(&root)
                    .unwrap_or_default()
            );
            let node =
                serde_json::from_str::<Definition>(&std::fs::read_to_string(e.path()).unwrap())
                    .unwrap();
            assert!(node
                .sources
                .iter()
                .all(|s| s.r#type != crate::ValueType::Other));
            assert!(node
                .targets
                .iter()
                .all(|t| t.type_bounds.iter().all(|t| *t != crate::ValueType::Other)));
        }
    }
}
