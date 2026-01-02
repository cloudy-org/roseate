use std::hash::Hash;

use serde::{Serialize, Deserialize};

use crate::config::models::ui::controls::Controls;

pub mod controls;
pub(self) use super::true_default;

#[derive(Serialize, Deserialize, Default, Hash, Clone)]
pub struct UI {
    #[serde(default)]
    pub controls: Controls,
    #[serde(default)]
    pub viewport: Viewport,
    #[serde(default)]
    pub selection_menu: SelectionMenu,
    #[serde(default)]
    pub info_panel: InfoPanel
}


#[derive(Serialize, Deserialize, Clone)]
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


#[derive(Serialize, Deserialize, Hash, Clone)]
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

#[derive(Serialize, Deserialize, Hash)]
pub struct InfoPanel {
    #[serde(default = "super::true_default")]
    pub show_location: bool
}

impl Default for InfoPanel {
    fn default() -> Self {
        Self {
            show_location: true
        }
    }
}
