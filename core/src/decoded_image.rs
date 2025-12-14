use crate::{colour_type::ImageColourType, format::ImageFormat};

pub type ImageSize = (u32, u32);
pub type Pixels<Channel = u8> = Vec<Channel>;

#[derive(Debug)]
pub enum DecodedImageContent {
    // NOTE: we do not support outputting channels higher than a u8 yet so 
    // there's no point of storing a decoded image in RAM as anything bigger than a u8
    Static(Pixels<u8>),
    Animated(Vec<(Pixels<u8>, f32)>),
}

pub struct DecodedImage {
    pub size: ImageSize,
    pub content: DecodedImageContent,
    pub colour_type: ImageColourType,
    pub image_format: ImageFormat,
}

impl DecodedImage {
    pub fn new(
        content: DecodedImageContent,
        colour_type: ImageColourType,
        image_format: ImageFormat,
        size: ImageSize
    ) -> Self {
        Self {
            content,
            colour_type,
            image_format,
            size
        }
    }
}