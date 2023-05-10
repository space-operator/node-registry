use serde::{Serialize, Deserialize};

pub mod initialize;
pub mod add_config_lines;

/// Config line struct for storing asset (NFT) data pre-mint.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConfigLine {
    /// Name of the asset.
    pub name: String,
    /// URI to JSON metadata.
    pub uri: String,
}

// implement from ConfigLine mpl_candy_machine_core::ConfigLine
impl From<ConfigLine> for mpl_candy_machine_core::ConfigLine {
    fn from(config_line: ConfigLine) -> Self {
        Self {
            name: config_line.name,
            uri: config_line.uri,
        }
    }
}

