use serde::{Serialize, Deserialize};

use crate::image::backends::ImageProcessingBackend;

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
    pub fn get_image_processing_backend(&self) -> ImageProcessingBackend {
        match &self.image_processing_backend {
            Some(backend_id) => {
                match backend_id.as_str() {
                    "image-rs" => ImageProcessingBackend::ImageRS,
                    "zune-image" => ImageProcessingBackend::ZuneImage,
                    "roseate" => ImageProcessingBackend::Roseate,
                    _ => ImageProcessingBackend::ImageRS
                }
            },
            None => ImageProcessingBackend::ImageRS,
        }
    }
}


#[derive(Serialize, Deserialize, Hash)]
pub struct OverrideMonitorSize {
    pub width: u32,
    pub height: u32
}