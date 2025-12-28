use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Default, Hash)]
pub struct Misc {
    #[serde(default = "super::none_default")]
    pub override_monitor_size: Option<OverrideMonitorSize>,
    #[serde(default)]
    pub experimental: Experimental,
}


#[derive(Serialize, Deserialize, Hash)]
pub struct Experimental {}

impl Default for Experimental {
    fn default() -> Self {
        Self {}
    }
}


#[derive(Serialize, Deserialize, Hash)]
pub struct OverrideMonitorSize {
    pub width: u32,
    pub height: u32
}