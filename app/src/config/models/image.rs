use serde::{Deserialize, Serialize};
use crate::{config::models::image_optimisations::ImageOptimisations, image::backend::DefaultDecodingBackend};
use std::hash::Hash;

#[derive(Serialize, Deserialize, Default, Hash, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct Image {
    #[serde(alias = "optimizations")]
    pub optimisations: ImageOptimisations,
    pub backend: Backend,
}

#[derive(Serialize, Deserialize, Hash, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct Backend {
    pub decoder: String,
}

impl Default for Backend {
    fn default() -> Self {
        Self {
            decoder: String::from("image-rs")
        }
    }
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