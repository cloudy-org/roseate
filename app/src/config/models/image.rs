use serde::{Deserialize, Serialize};
use crate::{config::models::image_optimizations::ImageOptimizations, image::backend::DefaultDecodingBackend};
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
    #[serde(default = "decoder_default")]
    pub decoder: String,
}

impl Backend {
    pub fn get_decoding_backend(&self) -> DefaultDecodingBackend {
        match self.decoder.as_str() {
            "image-rs" => DefaultDecodingBackend::ImageRS,
            "zune-image" => DefaultDecodingBackend::ZuneImage,
            _ => DefaultDecodingBackend::ImageRS
        }
    }
}

fn decoder_default() -> String {
    String::from("image-rs")
}


#[derive(Serialize, Deserialize, Default, Hash, Clone)]
pub struct ImageLoading {
    #[serde(default)]
    pub initial: InitialSettings,
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