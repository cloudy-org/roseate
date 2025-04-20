use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Default)]
pub struct Misc {
    #[serde(default = "super::none_default")]
    pub override_monitor_size: Option<OverrideMonitorSize>,
    #[serde(default)]
    pub experimental: Experimental,
}


#[derive(Serialize, Deserialize)]
pub struct Experimental {
    #[serde(default = "super::false_default")]
    pub use_fast_roseate_backend: bool,
    #[serde(default = "super::false_default")]
    pub use_dynamic_sampling_optimization: bool
}

impl Default for Experimental {
    fn default() -> Self {
        Self {
            use_fast_roseate_backend: false,
            use_dynamic_sampling_optimization: false
        }
    }
}


#[derive(Serialize, Deserialize)]
pub struct OverrideMonitorSize {
    pub width: u32,
    pub height: u32
}