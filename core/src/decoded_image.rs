use crate::{colour_type::ImageColourType, format::ImageFormat, metadata::ImageMetadata};

pub type ImageSize = (u32, u32);
pub type Pixels<Channel = u8> = Vec<Channel>;

#[derive(Debug)]
pub enum DecodedImageContent {
    // NOTE: we do not support outputting channels higher than a u8 yet so 
    // there's no point of storing a decoded image in RAM as anything bigger than a u8
    Static(Pixels),
    Animated(Vec<(Pixels, f32)>),
}

#[derive(Clone)]
pub struct DecodedImageInfo {
    pub size: ImageSize,
    pub format: ImageFormat,
    pub colour_type: ImageColourType,
    pub metadata: ImageMetadata,
}

// TODO: Pass more info about the image itself like EXIF tags 
// and what camera was used from the decoder to DecodedImage.
pub struct DecodedImage {
    pub info: DecodedImageInfo,
    pub content: DecodedImageContent,
}

impl DecodedImage {
    pub fn new(
        info: DecodedImageInfo,
        content: DecodedImageContent,
    ) -> Self {
        Self {
            info,
            content
        }
    }
}