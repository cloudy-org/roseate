use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Hash, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct Misc {
    pub show_detailed_errors: bool,
    pub override_monitor_size: Option<OverrideMonitorSize>,
    pub experimental: Experimental,
}

impl Default for Misc {
    fn default() -> Self {
        Self {
            show_detailed_errors: true,
            override_monitor_size: None,
            experimental: Experimental::default()
        }
    }
}


#[derive(Serialize, Deserialize, Hash, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct Experimental {
    pub show_ui_modes_popup: bool
}

impl Default for Experimental {
    fn default() -> Self {
        Self {
            // TODO: move to some alternative config outside config.toml.
            // If the whole config fails to parse or something 
            // I don't want users being bombarded with popups
            show_ui_modes_popup: true
        }
    }
}


#[derive(Serialize, Deserialize, Hash, Clone, Debug, PartialEq)]
pub struct OverrideMonitorSize {
    pub width: u32,
    pub height: u32
}