use serde::{Deserialize, Serialize};
use crate::{config::models::image_optimizations::ImageOptimizations};
use std::hash::Hash;

#[derive(Serialize, Deserialize, Default, Hash)]
pub struct Image {
    #[serde(default)]
    pub loading: ImageLoading,
    #[serde(default)]
    pub optimizations: ImageOptimizations
}


#[derive(Serialize, Deserialize, Default, Hash)]
pub struct ImageLoading {
    #[serde(default)]
    pub gui: LoadingGUISettings,
    #[serde(default)]
    pub initial: InitialSettings,
}


#[derive(Serialize, Deserialize, Hash)]
pub struct LoadingGUISettings {
    #[serde(default = "super::true_default")]
    pub lazy_loading: bool,
}

impl Default for LoadingGUISettings {
    fn default() -> Self {
        Self {
            lazy_loading: true,
        }
    }
}


#[derive(Serialize, Deserialize, Hash)]
pub struct InitialSettings {
    #[serde(default = "super::false_default")]
    pub lazy_loading: bool,
}

impl Default for InitialSettings {
    fn default() -> Self {
        Self {
            lazy_loading: false
        }
    }
}