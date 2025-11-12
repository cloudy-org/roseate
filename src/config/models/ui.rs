use std::hash::Hash;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Default, Hash)]
pub struct UI {
    pub magnification_panel: MagnificationPanel,
    pub viewport: Viewport
}


#[derive(Serialize, Deserialize, Hash)]
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
    #[serde(default = "super::true_default")]
    pub zoom_into_cursor: bool
}

impl Hash for Viewport {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        ((self.padding * 100.0) as u32).hash(state);
        self.zoom_into_cursor.hash(state);
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            padding: ui_padding(),
            zoom_into_cursor: true
        }
    }
}

fn ui_padding() -> f32 {
    2.0
}