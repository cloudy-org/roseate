use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Default, Hash, Clone)]
pub struct Misc {
    #[serde(default = "super::true_default")]
    pub show_detailed_errors: bool,
    #[serde(default = "super::none_default")]
    pub override_monitor_size: Option<OverrideMonitorSize>,
    #[serde(default)]
    pub experimental: Experimental,
}


#[derive(Serialize, Deserialize, Hash, Clone)]
pub struct Experimental {
    #[serde(default = "super::true_default")]
    pub show_ui_modes_popup: bool
}

impl Default for Experimental {
    fn default() -> Self {
        Self {
            // TODO: move to some alternative config outside config.toml.
            // If the whole config fails to parse or something 
            // I don't want users being bombarded with popups
            show_ui_modes_popup: false
        }
    }
}


#[derive(Serialize, Deserialize, Hash, Clone)]
pub struct OverrideMonitorSize {
    pub width: u32,
    pub height: u32
}