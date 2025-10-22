use cirrus_config::v1::config::CConfig;
use serde::{Serialize, Deserialize};

use super::models::{image::Image, key_binds::KeyBinds, misc::Misc, ui::UI};

#[derive(Serialize, Deserialize, Default, Hash)]
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