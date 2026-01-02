use serde::{Deserialize, Serialize};
use crate::{config::models::image_optimizations::ImageOptimizations, image::backend::DecodingBackend};
use std::hash::Hash;

#[derive(Serialize, Deserialize, Default, Hash, Clone)]
pub struct Image {
    #[serde(default)]
    pub loading: ImageLoading,
    #[serde(default)]
    pub optimizations: ImageOptimizations,
    #[serde(default)]
    pub backend: Backend,
}

#[derive(Serialize, Deserialize, Default, Hash, Clone)]
pub struct Backend {
    #[serde(default = "super::none_default")]
    decoder: Option<String>,
}

impl Backend {
    pub fn get_decoding_backend(&self) -> DecodingBackend {
        match &self.decoder {
            Some(backend_id) => {
                match backend_id.as_str() {
                    "image-rs" => DecodingBackend::ImageRS,
                    "zune-image" => DecodingBackend::ZuneImage,
                    _ => DecodingBackend::ImageRS
                }
            },
            None => DecodingBackend::ImageRS,
        }
    }
}


#[derive(Serialize, Deserialize, Default, Hash, Clone)]
pub struct ImageLoading {
    #[serde(default)]
    pub gui: LoadingGUISettings,
    #[serde(default)]
    pub initial: InitialSettings,
}


#[derive(Serialize, Deserialize, Hash, Clone)]
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


#[derive(Serialize, Deserialize, Hash, Clone)]
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