use cirrus_config::config::CConfig;
use serde::{Serialize, Deserialize};

use crate::config::models::ui::{HomeMenu, controls::Controls};

use super::models::{image::Image, key_binds::KeyBinds, misc::Misc, ui::UI};

#[derive(Serialize, Deserialize, Hash, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct Config {
    pub version: u8,
    pub image: Image,
    pub ui: UI,
    pub key_binds: KeyBinds,
    pub misc: Misc,
}

impl CConfig for Config {}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: 1,

            image: Image::default(),
            ui: UI::default(),
            key_binds: KeyBinds::default(),
            misc: Misc::default()
        }
    }
}

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

// TODO: move this into tests/test_config.rs
// I think this will require making roseate app a lib so we can export some stuff
#[cfg(test)]
mod tests {
    use crate::{TEMPLATE_CONFIG_TOML_STRING, config::config::Config, error::Result};

    #[test]
    fn test_config_validity() -> Result<()> {
        let default_config = Config::default();

        // The template should deserialize without trouble.
        let template_config: Config = toml::from_str(TEMPLATE_CONFIG_TOML_STRING).unwrap();

        // The template config should match exactly with the default values of our Config struct.
        assert_eq!(default_config, template_config);

        Ok(())
    }
}