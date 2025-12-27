use std::hash::Hash;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Default, Hash)]
pub struct UI {
    #[serde(default)]
    pub magnification_panel: MagnificationPanel,
    #[serde(default)]
    pub viewport: Viewport,
    #[serde(default)]
    pub selection_menu: SelectionMenu
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
    pub zoom_into_cursor: bool,
    #[serde(default = "super::true_default")]
    pub fit_to_window: bool,
    #[serde(default = "super::true_default")]
    pub animate_fit_to_window: bool,
    #[serde(default = "super::true_default")]
    pub animate_reset: bool,
}

impl Hash for Viewport {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        ((self.padding * 100.0) as u32).hash(state);
        self.zoom_into_cursor.hash(state);
        self.fit_to_window.hash(state);
        self.animate_fit_to_window.hash(state);
        self.animate_reset.hash(state);
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            padding: ui_padding(),
            zoom_into_cursor: true,
            fit_to_window: true,
            animate_fit_to_window: true,
            animate_reset: true
        }
    }
}

fn ui_padding() -> f32 {
    2.0
}


#[derive(Serialize, Deserialize, Hash)]
pub struct SelectionMenu {
    // #[serde(default = "super::none_default")]
    // pub mode: Option<String>,

    #[serde(default = "super::true_default")]
    pub show_open_image_button: bool
}

impl Default for SelectionMenu {
    fn default() -> Self {
        Self {
            show_open_image_button: true
        }
    }
}