use image::{ImageBuffer, Luma, LumaA, Rgb, Rgba};

use crate::{colour_type::ImageColourType, decoded_image::{ImageSize, Pixels}, error::Result};


/// Wrapper around different variants of image-rs image buffers.
pub enum BufferImage {
    // BufferImage doesn't contain u16 and f32 variants 
    // as a DecodedImage will always contain u8 pixels.
    Grey8(ImageBuffer::<Luma<u8>, Vec<u8>>),
    GreyA8(ImageBuffer::<LumaA<u8>, Vec<u8>>),
    Rgb8(ImageBuffer::<Rgb<u8>, Vec<u8>>),
    Rgba8(ImageBuffer::<Rgba<u8>, Vec<u8>>),
}

impl BufferImage {
    pub fn from_u8_pixels(pixels: Pixels<u8>, size: ImageSize, colour_type: ImageColourType) -> Result<Self> {
        let (width, height) = size;

        // TODO: handle result properly
        Ok(
            match colour_type {
                ImageColourType::Grey8 | ImageColourType::Grey16 | ImageColourType::Grey32F => BufferImage::Grey8(
                    ImageBuffer::from_raw(width, height, pixels).unwrap()
                ),
                ImageColourType::GreyA8 | ImageColourType::GreyA16 | ImageColourType::GreyA32F => BufferImage::GreyA8(
                    ImageBuffer::from_raw(width, height, pixels).unwrap()
                ),
                ImageColourType::Rgb8 | ImageColourType::Rgb16 | ImageColourType::Rgb32F => BufferImage::Rgb8(
                    ImageBuffer::from_raw(width, height, pixels).unwrap()
                ),
                ImageColourType::Rgba8 | ImageColourType::Rgba16 | ImageColourType::Rgba32F => BufferImage::Rgba8(
                    ImageBuffer::from_raw(width, height, pixels).unwrap()
                ),
            }
        )
    }

    pub fn to_u8_pixels(self) -> (Pixels<u8>, ImageSize, ImageColourType) {
        let size = match &self {
            BufferImage::Grey8(image_buffer) => image_buffer.dimensions(),
            BufferImage::GreyA8(image_buffer) => image_buffer.dimensions(),
            BufferImage::Rgb8(image_buffer) => image_buffer.dimensions(),
            BufferImage::Rgba8(image_buffer) => image_buffer.dimensions(),
        };

        match self {
            BufferImage::Grey8(image_buffer) => {
                (image_buffer.into_raw(), size, ImageColourType::Grey8)
            },
            BufferImage::GreyA8(image_buffer) => {
                (image_buffer.into_raw(), size, ImageColourType::GreyA8)
            },
            BufferImage::Rgb8(image_buffer) => {
                (image_buffer.into_raw(), size, ImageColourType::Rgb8)
            },
            BufferImage::Rgba8(image_buffer) => {
                (image_buffer.into_raw(), size, ImageColourType::Rgba8)
            },
        }
    }
}