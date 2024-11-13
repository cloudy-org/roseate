use std::fs;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct ImageSettings {
    pub initial: InitialSettings,
    pub dynamic: DynamicSettings,
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
pub struct Keybinds {
    pub info_box_toggle: String,
    pub image_reset_pos: String,
    pub magnification_panel_toggle: String
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub version: i8,
    pub image: ImageSettings,
    pub keybinds: Keybinds,
}

impl Config {
    pub fn new() -> Self {
        let local_dir = dirs::config_local_dir().expect("No config path found for your os!?");

        let roseate = local_dir.join("cloudy").join("roseate");

        if !roseate.exists() {
            if let Err(err) = fs::create_dir_all(roseate.clone()) {
                eprintln!("Unable to create config path: {}", err);
            };
        }

        let toml_file = roseate.join("config.toml");

        if toml_file.exists() {
            if let Ok(value) = fs::read_to_string(&toml_file) {
                if let Ok(config) = toml::from_str::<Config>(&value) {
                    return config;
                }
            }
        }

        let config = Config {
            version: 1,
            image: ImageSettings {
                initial: InitialSettings {
                    lazy_loading: false,
                },
                dynamic: DynamicSettings {
                    lazy_loading: true,
                },
            },
            keybinds: Keybinds {
                info_box_toggle: "I".to_string(),
                image_reset_pos: "R".to_string(),
                magnification_panel_toggle: "C".to_string(),
            },
        };
    
        let toml_string = toml::to_string(&config).expect("Failed to serialize the config");
        if let Err(err) = fs::write(toml_file, toml_string) {
            eprintln!("Unable to write default config to file: {}", err);
        }
    
        config
    }
}