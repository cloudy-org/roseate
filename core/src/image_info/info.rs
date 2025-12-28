use crate::{colour_type::ImageColourType, decoded_image::ImageSize, format::ImageFormat, image_info::metadata::ImageMetadata};

#[derive(Clone)]
pub struct ImageInfo {
    pub size: ImageSize,
    pub format: ImageFormat,
    pub colour_type: ImageColourType,
    pub metadata: ImageMetadata,
}