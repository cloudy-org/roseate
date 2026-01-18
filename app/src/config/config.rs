use cirrus_config::v1::config::CConfig;
use serde::{Serialize, Deserialize};

use crate::config::models::ui::{ImageInfo, SelectionMenu, Viewport, controls::Controls};

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
                        hide: false,
                        magnification: true,
                    },
                    viewport: Viewport::default(),
                    selection_menu: SelectionMenu {
                        show_open_image_button: true,
                    },
                    image_info: ImageInfo::default(),
                }
            },
            UIConfigMode::Minimalist => {
                self.ui = UI {
                    controls: Controls {
                        hide: true,
                        magnification: true,
                    },
                    viewport: Viewport::default(),
                    selection_menu: SelectionMenu {
                        show_open_image_button: false,
                    },
                    image_info: ImageInfo::default(),
                }
            },
        }
    }
}