use cirrus_config::config::CConfig;
use serde::{Serialize, Deserialize};

use crate::config::models::ui::{HomeMenu, controls::Controls};

use super::models::{image::Image, key_binds::KeyBinds, misc::Misc, ui::UI};

#[derive(Serialize, Deserialize, Default, Hash, Clone)]
pub struct Config {
    #[serde(default)]
    pub version: i8,
    #[serde(default)]
    pub image: Image,
    #[serde(default)]
    pub ui: UI,
    #[serde(default)]
    pub key_binds: KeyBinds,
    #[serde(default)]
    pub misc: Misc,
}

impl CConfig for Config {}

pub enum UIConfigMode {
    Standard,
    Minimalist,
}

impl Config {
    pub fn override_ui_config(&mut self, ui_mode: UIConfigMode) {
        match ui_mode {
            UIConfigMode::Standard => {
                self.ui = UI {
                    controls: Controls {
                        show: true,
                        ..self.ui.controls
                    },
                    viewport: self.ui.viewport.clone(),
                    home_menu: HomeMenu {
                        show_settings_button: true,
                        show_open_image_button: true,
                    },
                    image_info: self.ui.image_info.clone(),
                }
            },
            UIConfigMode::Minimalist => {
                self.ui = UI {
                    controls: Controls {
                        show: false,
                        ..self.ui.controls
                    },
                    viewport: self.ui.viewport.clone(),
                    home_menu: HomeMenu {
                        show_settings_button: false,
                        show_open_image_button: false,
                    },
                    image_info: self.ui.image_info.clone(),
                }
            },
        }
    }
}