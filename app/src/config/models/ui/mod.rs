use std::hash::Hash;

use serde::{Serialize, Deserialize};

use crate::config::models::ui::controls::Controls;

pub mod controls;
pub(self) use super::{true_default, false_default};

#[derive(Serialize, Deserialize, Default, Hash, Clone)]
pub struct UI {
    #[serde(default)]
    pub controls: Controls,
    #[serde(default)]
    pub viewport: Viewport,
    #[serde(default)]
    pub home_menu: HomeMenu,
    #[serde(default)]
    pub image_info: ImageInfo
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
pub struct HomeMenu {
    // #[serde(default = "super::none_default")]
    // pub mode: Option<String>,

    #[serde(default = "super::true_default")]
    pub show_settings_button: bool,

    #[serde(default = "super::true_default")]
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


#[derive(Serialize, Deserialize, Hash, Clone)]
pub struct ImageInfo {
    #[serde(default = "super::true_default")]
    pub show_location: bool
}

impl Default for ImageInfo {
    fn default() -> Self {
        Self {
            show_location: true
        }
    }
}