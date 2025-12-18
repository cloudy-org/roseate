use std::fmt::Display;

use roseate_core::{backends::{backend::DecodeBackend, image_rs::ImageRSBackend}, reader::ImageReader};

use crate::error::Result;

pub enum DecodingBackend {
    /// Uses the image-rs rust crate for image decoding and 
    /// modifications. This is by far the most stable backend.
    ImageRS,
    /// Uses the zune-image rust crate for image decoding and modifications, 
    /// it's fast but it's implementation in roseate is currently experimental.
    ZuneImage,
}

impl Display for DecodingBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecodingBackend::ImageRS => write!(f, "image-rs"),
            DecodingBackend::ZuneImage => write!(f, "zune-image"),
        }
    }
}

impl DecodingBackend {
    pub fn init_decoder(&self, image_reader: ImageReader) -> Result<impl DecodeBackend> {
        let decoder = match self {
            DecodingBackend::ImageRS => ImageRSBackend::from_reader(image_reader)?,
            DecodingBackend::ZuneImage => todo!(),
        };

        Ok(decoder)
    }
}