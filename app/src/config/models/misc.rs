use serde::{Serialize, Deserialize};

use crate::image::backend::DecodingBackend;

#[derive(Serialize, Deserialize, Default, Hash)]
pub struct Misc {
    #[serde(default = "super::none_default")]
    pub override_monitor_size: Option<OverrideMonitorSize>,
    #[serde(default)]
    pub experimental: Experimental,
}


#[derive(Serialize, Deserialize, Hash)]
pub struct Experimental {
    #[serde(default = "super::none_default")]
    image_processing_backend: Option<String>,
}

impl Default for Experimental {
    fn default() -> Self {
        Self {
            image_processing_backend: None,
        }
    }
}

impl Experimental {
    pub fn get_decoding_backend(&self) -> DecodingBackend {
        match &self.image_processing_backend {
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


#[derive(Serialize, Deserialize, Hash)]
pub struct OverrideMonitorSize {
    pub width: u32,
    pub height: u32
}