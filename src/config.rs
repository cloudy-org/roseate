use std::{error::Error, fs};
use eframe::egui::TextBuffer;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct GUISettings {
    pub lazy_loading: bool,
}

#[derive(Serialize, Deserialize)]
pub struct InitialSettings {
    pub lazy_loading: bool,
}

#[derive(Serialize, Deserialize)]
pub struct DynamicSettings {
    pub lazy_loading: bool,
}

#[derive(Serialize, Deserialize)]
pub struct ImageLoading {
    pub gui: GUISettings,
    pub initial: InitialSettings,
    pub dynamic: DynamicSettings,
}

#[derive(Serialize, Deserialize)]
pub struct Image {
    pub loading: ImageLoading,
}

#[derive(Serialize, Deserialize)]
pub struct InfoBoxBinds {
    pub toggle: String,
}

#[derive(Serialize, Deserialize)]
pub struct ImageBinds {
    pub reset_pos: String,
}

#[derive(Serialize, Deserialize)]
pub struct UIControlsBinds {
    pub toggle: String,
}

#[derive(Serialize, Deserialize)]
pub struct Keybinds {
    pub info_box: InfoBoxBinds,
    pub image: ImageBinds,
    pub ui_controls: UIControlsBinds
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub version: i8,
    pub image: Image,
    pub keybinds: Keybinds,
}

impl Config {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let local_dir = dirs::config_local_dir()
            .expect("No config path found for your os!?");

        let config_dir_path = local_dir.join("cloudy").join("roseate");

        if !config_dir_path.exists() {
            fs::create_dir_all(&config_dir_path)?;
            if let Err(err) = fs::create_dir_all(&config_dir_path) {
                return Err(
                    format!("Unable to create config path: {}", err).into()
                );
            };
        }

        let toml_config_path = config_dir_path.join("config.toml");

        if toml_config_path.exists() {
            let value = fs::read_to_string(&toml_config_path)?;

            let config = toml::from_str::<Config>(&value)?;
            return Ok(config);
        }

        let result = fs::write(
            &toml_config_path, include_str!("../assets/config.template.toml")
        );

        match result {
            Ok(_) => Ok(
                toml::from_str(include_str!("../assets/config.template.toml"))
                    .expect("Failed to deserialize template toml file!")
                // I'm panicking here as if this fails to deserialize it's our fault!
                // Tests should be put in place to ensure this doesn't happen from our end.
                // 
                // TODO: Make a cargo test to confirm the config.template.toml 
                // deserializes without error. Then also add it as a github workflow.
            ),
            Err(error) => {
                Err(
                    format!(
                        "Unable to create toml config at '{}'! Defaulting to default config. Error: {}",
                        toml_config_path.to_string_lossy().as_str(), error
                    ).into()
                )
            }
        }
    }

    pub fn default() -> Self {
        Self {
            version: 1,
            image: Image {
                loading: ImageLoading {
                    gui: GUISettings {
                        lazy_loading: true
                    },
                    initial: InitialSettings {
                        lazy_loading: false,
                    },
                    dynamic: DynamicSettings {
                        lazy_loading: true,
                    },
                }
            },
            keybinds: Keybinds {
                info_box: InfoBoxBinds { toggle: "I".to_string() },
                image: ImageBinds { reset_pos: "R".to_string() },
                ui_controls: UIControlsBinds { toggle: "C".to_string() },
            },
        }
    }
}