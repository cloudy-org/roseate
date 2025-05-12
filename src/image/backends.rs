use std::fmt::Display;

use image::{DynamicImage, ImageBuffer, Rgb};

use super::image::{ImageRSImage, ImageSizeT};

pub enum ImageProcessingBackend {
    /// Uses the image-rs rust crate for decoding and image manipulation.
    /// This is by far the most stable backend.
    ImageRS,
    /// Uses zune-image rust crate for decoding but roseate's 
    /// custom methods for image manipulation.
    ZuneImage,
    /// Uses Roseate's custom backend for image processing and manipulation.
    /// The Roseate backend is very fast for downsampling images but is VERY EXPERIMENTAL 
    /// and currently has wonky downsampling hence images may have weird artifacts.
    Roseate
}

pub enum ImageDecodePipelineKind {
    ImageRS,
    ZuneImage
}

pub enum ImageDecodePipeline<'a> {
    ImageRS(&'a [u8]),
}

pub enum ModificationProcessingMeatKind {
    ImageRS,
    Roseate
}

pub enum ModificationProcessingMeat<'a> {
    ImageRS(&'a mut ImageRSImage),
    Roseate(&'a mut Vec<u8>, &'a mut ImageSizeT, bool)
}

impl ImageProcessingBackend {
    pub fn get_decode_pipeline(&self) -> ImageDecodePipelineKind {
        match self {
            ImageProcessingBackend::ImageRS => ImageDecodePipelineKind::ImageRS,
            ImageProcessingBackend::ZuneImage => ImageDecodePipelineKind::ZuneImage,
            ImageProcessingBackend::Roseate => ImageDecodePipelineKind::ZuneImage,
        }
    }

    pub fn get_modification_processing_meat(&self) -> ModificationProcessingMeatKind {
        match self {
            ImageProcessingBackend::ImageRS => ModificationProcessingMeatKind::ImageRS,
            ImageProcessingBackend::ZuneImage => ModificationProcessingMeatKind::Roseate,
            ImageProcessingBackend::Roseate => ModificationProcessingMeatKind::Roseate,
        }
    }
}

impl Display for ImageProcessingBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageProcessingBackend::ZuneImage => write!(f, "zune-image"),
            ImageProcessingBackend::ImageRS => write!(f, "image-rs"),
            ImageProcessingBackend::Roseate => write!(f, "roseate"),
        }
    }
}