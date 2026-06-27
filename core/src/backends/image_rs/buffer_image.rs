use image::{ImageBuffer, Luma, LumaA, Rgb, Rgba};

use crate::{colour_type::ImageColourType, decoded_image::ImageSize, error::Result, pixels::Pixels};

pub enum BufferImageVariant {
    Grey8(ImageBuffer::<Luma<u8>, Vec<u8>>),
    Grey16(ImageBuffer::<Luma<u16>, Vec<u16>>),
    Grey32F(ImageBuffer::<Luma<f32>, Vec<f32>>),

    GreyA8(ImageBuffer::<LumaA<u8>, Vec<u8>>),
    GreyA16(ImageBuffer::<LumaA<u16>, Vec<u16>>),
    GreyA32F(ImageBuffer::<LumaA<f32>, Vec<f32>>),

    Rgb8(ImageBuffer::<Rgb<u8>, Vec<u8>>),
    Rgb16(ImageBuffer::<Rgb<u16>, Vec<u16>>),
    Rgb32F(ImageBuffer::<Rgb<f32>, Vec<f32>>),

    Rgba8(ImageBuffer::<Rgba<u8>, Vec<u8>>),
    Rgba16(ImageBuffer::<Rgba<u16>, Vec<u16>>),
    Rgba32F(ImageBuffer::<Rgba<f32>, Vec<f32>>),
}

impl BufferImageVariant {
    pub fn from_pixels_and_colour_type(pixels: Pixels, size: ImageSize, colour_type: ImageColourType) -> Self {
        let (width, height) = size;

        // TODO: handle result properly
        match pixels {
            Pixels::U8(vec) => {
                match colour_type {
                    ImageColourType::Grey8 => Self::Grey8(ImageBuffer::from_raw(width, height, vec).unwrap()),
                    ImageColourType::GreyA8 => Self::GreyA8(ImageBuffer::from_raw(width, height, vec).unwrap()),
                    ImageColourType::Rgb8 => Self::Rgb8(ImageBuffer::from_raw(width, height, vec).unwrap()),
                    ImageColourType::Rgba8 => Self::Rgba8(ImageBuffer::from_raw(width, height, vec).unwrap()),
                    _ => unreachable!()
                }
            },
            Pixels::U16(vec) => {
                match colour_type {
                    ImageColourType::Grey16 => Self::Grey16(ImageBuffer::from_raw(width, height, vec).unwrap()),
                    ImageColourType::GreyA16 => Self::GreyA16(ImageBuffer::from_raw(width, height, vec).unwrap()),
                    ImageColourType::Rgb16 => Self::Rgb16(ImageBuffer::from_raw(width, height, vec).unwrap()),
                    ImageColourType::Rgba16 => Self::Rgba16(ImageBuffer::from_raw(width, height, vec).unwrap()),
                    _ => unreachable!()
                }
            },
            Pixels::F32(vec) => {
                match colour_type {
                    ImageColourType::Grey32F => Self::Grey32F(ImageBuffer::from_raw(width, height, vec).unwrap()),
                    ImageColourType::GreyA32F => Self::GreyA32F(ImageBuffer::from_raw(width, height, vec).unwrap()),
                    ImageColourType::Rgb32F => Self::Rgb32F(ImageBuffer::from_raw(width, height, vec).unwrap()),
                    ImageColourType::Rgba32F => Self::Rgba32F(ImageBuffer::from_raw(width, height, vec).unwrap()),
                    _ => unreachable!()
                }
            },
        }
    }
}

/// Wrapper around different variants of image-rs image buffers.
pub struct BufferImage {
    // TODO: remove size once done with support higher bit depths pr
    pub size: ImageSize,
    pub colour_type: ImageColourType,
    pub variant: BufferImageVariant,
}

impl BufferImage {
    pub fn from_pixels(pixels: Pixels, size: ImageSize, colour_type: ImageColourType) -> Result<Self> {
        // TODO: handle result when it has one
        let variant = BufferImageVariant::from_pixels_and_colour_type(
            pixels,
            size,
            colour_type
        );

        Ok(Self {
            size,
            colour_type,
            variant
        })
    }

    pub fn to_pixels(self) -> (Pixels, ImageSize, ImageColourType) {
        let (image_size, image_pixels) = match self.variant {
            BufferImageVariant::Grey8(image_buffer) => (image_buffer.dimensions(), Pixels::U8(image_buffer.into_raw())),
            BufferImageVariant::GreyA8(image_buffer) => (image_buffer.dimensions(), Pixels::U8(image_buffer.into_raw())),
            BufferImageVariant::Rgb8(image_buffer) => (image_buffer.dimensions(), Pixels::U8(image_buffer.into_raw())),
            BufferImageVariant::Rgba8(image_buffer) => (image_buffer.dimensions(), Pixels::U8(image_buffer.into_raw())),

            BufferImageVariant::Grey16(image_buffer) => (image_buffer.dimensions(), Pixels::U16(image_buffer.into_raw())),
            BufferImageVariant::GreyA16(image_buffer) => (image_buffer.dimensions(), Pixels::U16(image_buffer.into_raw())),
            BufferImageVariant::Rgb16(image_buffer) => (image_buffer.dimensions(), Pixels::U16(image_buffer.into_raw())),
            BufferImageVariant::Rgba16(image_buffer) => (image_buffer.dimensions(), Pixels::U16(image_buffer.into_raw())),

            BufferImageVariant::Grey32F(image_buffer) => (image_buffer.dimensions(), Pixels::F32(image_buffer.into_raw())),
            BufferImageVariant::GreyA32F(image_buffer) => (image_buffer.dimensions(), Pixels::F32(image_buffer.into_raw())),
            BufferImageVariant::Rgb32F(image_buffer) => (image_buffer.dimensions(), Pixels::F32(image_buffer.into_raw())),
            BufferImageVariant::Rgba32F(image_buffer) => (image_buffer.dimensions(), Pixels::F32(image_buffer.into_raw())),
        };

        (
            image_pixels,
            image_size,
            self.colour_type
        )
    }
}