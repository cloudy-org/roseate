use std::hash::Hash;

use serde::{Serialize, Deserialize};

use crate::config::models::ui::controls::Controls;

pub mod controls;

#[derive(Serialize, Deserialize, Default, Hash, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct UI {
    pub controls: Controls,
    pub viewport: Viewport,
    pub home_menu: HomeMenu,
    pub image_info: ImageInfo
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct Viewport {
    pub padding: f32,
    pub zoom_into_cursor: bool,
    pub fit_to_window: bool,
    pub animate_fit_to_window: bool,
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
            padding: 2.0,
            zoom_into_cursor: true,
            fit_to_window: true,
            animate_fit_to_window: true,
            animate_reset: true
        }
    }
}

#[derive(Serialize, Deserialize, Hash, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct HomeMenu {
    // #[serde(default = "super::none_default")]
    // pub mode: Option<String>,

    pub show_settings_button: bool,
    pub show_open_image_button: bool
}

impl Default for HomeMenu {
    fn default() -> Self {
        Self {
            show_settings_button: true,
            show_open_image_button: true
        }
    }
}


#[derive(Serialize, Deserialize, Hash, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct ImageInfo {
    pub show_location: bool
}

impl Default for ImageInfo {
    fn default() -> Self {
        Self {
            show_location: true
        }
    }
}