use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Default)]
pub struct UI {
    pub magnification_panel: MagnificationPanel,
    pub viewport: Viewport
}


#[derive(Serialize, Deserialize)]
pub struct MagnificationPanel {
    #[serde(default = "super::true_default")]
    pub enabled_default: bool,
}

impl Default for MagnificationPanel {
    fn default() -> Self {
        Self {
            enabled_default: false
        }
    }
}


#[derive(Serialize, Deserialize)]
pub struct Viewport {
    #[serde(default = "ui_padding")]
    pub padding: f32,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            padding: ui_padding()
        }
    }
}

fn ui_padding() -> f32 {
    2.0
}