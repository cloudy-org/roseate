use log::debug;
use std::{error::Error, fs};
use eframe::egui::TextBuffer;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct GUISettings {
    #[serde(default)]
    pub lazy_loading: bool,
}

#[derive(Serialize, Deserialize)]
pub struct InitialSettings {
    #[serde(default)]
    pub lazy_loading: bool,
}

#[derive(Serialize, Deserialize)]
pub struct DynamicSettings {
    #[serde(default)]
    pub lazy_loading: bool,
}

#[derive(Serialize, Deserialize)]
pub struct ImageLoading {
    #[serde(default)]
    pub gui: GUISettings,
    #[serde(default)]
    pub initial: InitialSettings,
    #[serde(default)]
    pub dynamic: DynamicSettings,
}

#[derive(Serialize, Deserialize)]
pub struct Image {
    #[serde(default)]
    pub loading: ImageLoading,
}

#[derive(Serialize, Deserialize)]
pub struct InfoBoxBinds {
    #[serde(default)]
    pub toggle: String,
}

#[derive(Serialize, Deserialize)]
pub struct ImageBinds {
    #[serde(default)]
    pub reset_pos: String,
}

#[derive(Serialize, Deserialize)]
pub struct UIControlsBinds {
    #[serde(default)]
    pub toggle: String,
}

#[derive(Serialize, Deserialize)]
pub struct Keybinds {
    #[serde(default)]
    pub info_box: InfoBoxBinds,
    #[serde(default)]
    pub image: ImageBinds,
    #[serde(default)]
    pub ui_controls: UIControlsBinds
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub version: i8,
    #[serde(default)]
    pub image: Image,
    #[serde(default)]
    pub keybinds: Keybinds,
}

impl Config {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        debug!("Finding operating system's configuration local directory...");
        let local_config_dir = match dirs::config_local_dir() {
            Some(dir) => dir,
            None => {
                return Err("No config path was found for your OS!?".into());
            }
        };

        let roseate_config_dir_path = local_config_dir.join("cloudy").join("roseate");

        if !roseate_config_dir_path.exists() {
            debug!("Creating config directory for roseate...");
            if let Err(err) = fs::create_dir_all(&roseate_config_dir_path) {
                return Err(
                    format!("Unable to create config path: {}", err).into()
                );
            };

            debug!("Config directory created!");
        }

        let toml_config_path = roseate_config_dir_path.join("config.toml");

        if toml_config_path.exists() {
            debug!("Reading and applying config file...");
            let value = fs::read_to_string(&toml_config_path)?;

            let config = toml::from_str::<Config>(&value)?;
            return Ok(config);
        }

        debug!(
            "Reading template config and creating config file at '{}'...", 
            &toml_config_path.to_string_lossy().as_str()
        );
        let result = fs::write(
            &toml_config_path, include_bytes!("../assets/config.template.toml")
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
}

impl Default for GUISettings {
    fn default() -> Self {
        Self {
            lazy_loading: true
        }
    }
}

impl Default for InitialSettings {
    fn default() -> Self {
        Self {
            lazy_loading: false
        }
    }
}

impl Default for DynamicSettings {
    fn default() -> Self {
        Self {
            lazy_loading: true
        }
    }
}

impl Default for ImageLoading {
    fn default() -> Self {
        Self {
            gui: GUISettings::default(),
            initial: InitialSettings::default(),
            dynamic: DynamicSettings::default()
        }
    }
}

impl Default for Image {
    fn default() -> Self {
        Self {
            loading: ImageLoading::default()
        }
    }
}

impl Default for InfoBoxBinds {
    fn default() -> Self {
        Self {
            toggle: "I".to_string()
        }
    }
}

impl Default for ImageBinds {
    fn default() -> Self {
        Self {
            reset_pos: "R".to_string()
        }
    }
}

impl Default for UIControlsBinds {
    fn default() -> Self {
        Self {
            toggle: "C".to_string()
        }
    }
}

impl Default for Keybinds {
    fn default() -> Self {
        Self {
            info_box: InfoBoxBinds::default(),
            image: ImageBinds::default(),
            ui_controls: UIControlsBinds::default(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: 1,
            image: Image::default(),
            keybinds: Keybinds::default()
        }
    }
}