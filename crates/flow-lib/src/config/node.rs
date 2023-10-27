//! Note: only add fields that are needed in backend.
//!
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Definition {
    pub r#type: super::CommandType,
    pub data: Data,
    pub sources: Vec<Source>,
    pub targets: Vec<Target>,
    #[serde(default)]
    pub permissions: Permissions,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct Permissions {
    #[serde(default)]
    pub user_tokens: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Data {
    pub node_id: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Source {
    pub name: String,
    pub r#type: super::ValueType,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Target {
    pub name: String,
    pub type_bounds: Vec<super::ValueType>,
    pub required: bool,
    pub passthrough: bool,
}

#[cfg(test)]
mod tests {
    use walkdir::WalkDir;

    use super::*;

    #[test]
    fn test_parse_all() {
        let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../node-definitions");
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
            let _node =
                serde_json::from_str::<Definition>(&std::fs::read_to_string(e.path()).unwrap())
                    .unwrap();
            /*
            assert!(node
                .sources
                .iter()
                .all(|s| s.r#type != crate::ValueType::Other));
            assert!(node
                .targets
                .iter()
                .all(|t| t.type_bounds.iter().all(|t| *t != crate::ValueType::Other)));
            */
        }
    }
}
