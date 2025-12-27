use image::{ImageBuffer, Luma, LumaA, Rgb, Rgba};

use crate::{colour_type::ImageColourType, decoded_image::{DecodedImageInfo, Pixels}, error::Result};

pub enum BufferImageVariant {
    // BufferImage doesn't contain u16 and f32 variants 
    // as a DecodedImage will always contain u8 pixels.
    Grey8(ImageBuffer::<Luma<u8>, Vec<u8>>),
    GreyA8(ImageBuffer::<LumaA<u8>, Vec<u8>>),
    Rgb8(ImageBuffer::<Rgb<u8>, Vec<u8>>),
    Rgba8(ImageBuffer::<Rgba<u8>, Vec<u8>>),
}

/// Wrapper around different variants of image-rs image buffers.
pub struct BufferImage {
    pub info: DecodedImageInfo,
    pub variant: BufferImageVariant,
}

impl BufferImage {
    pub fn from_u8_pixels(pixels: Pixels<u8>, decoded_image_info: DecodedImageInfo) -> Result<Self> {
        let (width, height) = decoded_image_info.size;

        // TODO: handle result properly
        let variant = match decoded_image_info.colour_type {
            ImageColourType::Grey8 | ImageColourType::Grey16 | ImageColourType::Grey32F => BufferImageVariant::Grey8(
                ImageBuffer::from_raw(width, height, pixels).unwrap()
            ),
            ImageColourType::GreyA8 | ImageColourType::GreyA16 | ImageColourType::GreyA32F => BufferImageVariant::GreyA8(
                ImageBuffer::from_raw(width, height, pixels).unwrap()
            ),
            ImageColourType::Rgb8 | ImageColourType::Rgb16 | ImageColourType::Rgb32F => BufferImageVariant::Rgb8(
                ImageBuffer::from_raw(width, height, pixels).unwrap()
            ),
            ImageColourType::Rgba8 | ImageColourType::Rgba16 | ImageColourType::Rgba32F => BufferImageVariant::Rgba8(
                ImageBuffer::from_raw(width, height, pixels).unwrap()
            ),
        };

        Ok(Self {
            info: decoded_image_info,
            variant
        })
    }

    pub fn to_u8_pixels(self) -> (Pixels<u8>, DecodedImageInfo) {
        let size: (u32, u32) = match &self.variant {
            BufferImageVariant::Grey8(image_buffer) => image_buffer.dimensions(),
            BufferImageVariant::GreyA8(image_buffer) => image_buffer.dimensions(),
            BufferImageVariant::Rgb8(image_buffer) => image_buffer.dimensions(),
            BufferImageVariant::Rgba8(image_buffer) => image_buffer.dimensions(),
        };

        let mut decoded_image_info = self.info;
        decoded_image_info.size = size;

        match self.variant {
            BufferImageVariant::Grey8(image_buffer) => {
                (image_buffer.into_raw(), decoded_image_info)
            },
            BufferImageVariant::GreyA8(image_buffer) => {
                (image_buffer.into_raw(), decoded_image_info)
            },
            BufferImageVariant::Rgb8(image_buffer) => {
                (image_buffer.into_raw(), decoded_image_info)
            },
            BufferImageVariant::Rgba8(image_buffer) => {
                (image_buffer.into_raw(), decoded_image_info)
            },
        }
    }
}