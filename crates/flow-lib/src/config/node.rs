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
    pub node_definition_version: String,
    pub unique_id: String,
    pub node_id: String,
    pub version: String,
    pub display_name: String,
    pub description: String,
    pub tags: Option<Vec<String>>,
    pub related_to: Option<Vec<RelatedTo>>,
    pub resources: Option<Resources>,
    pub usage: Option<Usage>,
    pub authors: Option<Vec<Author>>,
    pub design: Option<Design>,
    pub options: Option<serde_json::Value>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct RelatedTo {
    pub id: String,
    pub r#type: String,
    pub relationship: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Resources {
    pub source_code_url: String,
    pub documentation_url: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Usage {
    pub license: String,
    pub license_url: String,
    pub pricing: Pricing,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Pricing {
    pub currency: String,
    pub purchase_price: u64,
    pub price_per_run: u64,
    pub custom: Option<CustomPricing>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct CustomPricing {
    pub unit: String,
    pub value: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Author {
    pub name: String,
    pub contact: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Design {
    pub width: u64,
    pub height: u64,
    pub icon_url: String,
    #[serde(rename = "backgroundColor")]
    pub background_color: String,
    #[serde(rename = "backgroundColorDark")]
    pub background_color_dark: String,
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
