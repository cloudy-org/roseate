use crate::{colour_type::ImageColourType, format::ImageFormat, image_info::{info::ImageInfo, metadata::ImageMetadata}};

pub type ImageSize = (u32, u32);
pub type Pixels<Channel = u8> = Vec<Channel>;

#[derive(Debug)]
pub enum DecodedImageContent {
    // NOTE: we do not support outputting channels higher than a u8 yet so 
    // there's no point of storing a decoded image in RAM as anything bigger than a u8
    Static(Pixels),
    Animated(Vec<(Pixels, f32)>),
}

// TODO: Add back size to decoded image
pub struct DecodedImage {
    pub info: ImageInfo,

    /// The actual size of the decoded image right now. 
    /// NOT the original image size (see `info.size` for that).
    pub size: ImageSize,
    pub colour_type: ImageColourType,
    pub content: DecodedImageContent,
}

impl DecodedImage {
    pub fn new(
        size: ImageSize,
        format: ImageFormat,
        colour_type: ImageColourType,
        metadata: ImageMetadata,
        content: DecodedImageContent,
    ) -> Self {
        let info = ImageInfo {
            size: size.clone(),
            format: format.clone(),
            colour_type: colour_type.clone(),
            metadata: metadata.clone(),
        };

        Self {
            info,
            size,
            colour_type,
            content
        }
    }
}