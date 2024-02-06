use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::states::app::{NodeLocation, VpnMode};

#[derive(Default, Serialize, Deserialize, Debug, Clone, TS)]
#[ts(export)]
pub enum UiTheme {
    Dark,
    #[default]
    Light,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone, TS)]
#[ts(export)]
pub struct AppData {
    pub monitoring: Option<bool>,
    pub autoconnect: Option<bool>,
    pub killswitch: Option<bool>,
    pub entry_location_selector: Option<bool>,
    pub ui_theme: Option<UiTheme>,
    pub ui_root_font_size: Option<u32>,
    pub vpn_mode: Option<VpnMode>,
    pub entry_node_location: Option<NodeLocation>,
    pub exit_node_location: Option<NodeLocation>,
}
