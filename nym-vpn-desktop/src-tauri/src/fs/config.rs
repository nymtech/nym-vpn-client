use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    /// Path pointing to an env configuration file describing the network
    pub env_config_file: Option<PathBuf>,
    pub default_entry_node_location_code: Option<String>,
    pub default_exit_node_location_code: Option<String>,
}
