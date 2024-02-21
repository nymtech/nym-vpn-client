use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    /// Path pointing to an env configuration file describing the network
    pub env_config_file: Option<PathBuf>,
    /// 2-letter country code for the default entry node location
    pub default_entry_node_location_code: Option<String>,
    /// 2-letter country code for the default exit node location
    pub default_exit_node_location_code: Option<String>,
}
