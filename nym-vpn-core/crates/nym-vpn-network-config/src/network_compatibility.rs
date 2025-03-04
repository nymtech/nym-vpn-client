use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkCompatibility {
    pub core: String,
    pub ios: String,
    pub macos: String,
    pub tauri: String,
    pub android: String,
}

impl fmt::Display for NetworkCompatibility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "core: {:?}\nios: {:?}\nmacos: {:?}\ntauri: {:?}\nandroid: {:?}",
            self.core, self.ios, self.macos, self.tauri, self.android
        )
    }
}
