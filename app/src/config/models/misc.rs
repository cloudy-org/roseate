use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Default, Hash)]
pub struct Misc {
    #[serde(default = "super::none_default")]
    pub override_monitor_size: Option<OverrideMonitorSize>,
    #[serde(default)]
    pub experimental: Experimental,
}


#[derive(Serialize, Deserialize, Hash)]
pub struct Experimental {
    #[serde(default = "super::true_default")]
    show_ui_modes_popup: bool
}

impl Default for Experimental {
    fn default() -> Self {
        Self {
            // if the whole config fails to parse or something 
            // I don't want users being bombarded with popups
            // 
            // TODO: move to some alternative config outside config.toml.
            show_ui_modes_popup: false
        }
    }
}


#[derive(Serialize, Deserialize, Hash)]
pub struct OverrideMonitorSize {
    pub width: u32,
    pub height: u32
}