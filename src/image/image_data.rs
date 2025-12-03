use std::sync::Arc;

use zune_image::codecs::qoi::zune_core::colorspace::ColorSpace;

use crate::error::{Error, Result};

use super::image::ImageSizeT;

#[derive(Clone)]
pub enum ImageColourType {
    GreyAlpha,
    Grey,
    RGB,
    RGBA
}

impl TryFrom<ColorSpace> for ImageColourType {
    type Error = Error;

    fn try_from(zune_image_colour_space: ColorSpace) -> Result<Self> {
        let result = match zune_image_colour_space {
            ColorSpace::RGB => Ok(ImageColourType::RGB),
            ColorSpace::RGBA => Ok(ImageColourType::RGBA),
            ColorSpace::YCbCr => Err(()),
            ColorSpace::Luma => Ok(ImageColourType::Grey),
            ColorSpace::LumaA => Ok(ImageColourType::Grey),
            ColorSpace::YCCK => Err(()),
            ColorSpace::CMYK => Err(()),
            ColorSpace::BGR => Err(()),
            ColorSpace::BGRA => Err(()),
            ColorSpace::Unknown => Ok(ImageColourType::RGB),
            ColorSpace::ARGB => todo!(),
            ColorSpace::HSL => Err(()),
            ColorSpace::HSV => Err(()),
            _ => Err(()),
        };

        match result {
            Ok(colour_type) => Ok(colour_type),
            Err(_) => {
                let error = Error::ImageFormatNotSupported(
                    None,
                    format!(
                        "The colour space '{:?}' the zune-image backend gave us is not supported in Roseate yet!",
                        zune_image_colour_space
                    )
                );

                Err(error)
            },
        }
    }
}

/// Use `ImageData::StaticBytes` if you wanna pass image decoding 
/// to egui and if this current image doesn't require modifications or 
/// modification is not possible, otherwise always use `ImageData::Pixels`.
#[derive(Clone)]
pub enum ImageData {
    Pixels((Arc<[u8]>, ImageSizeT, ImageColourType)),
    /// Use this for images that do not support decoding to pixels like SVGs 
    /// or for images that cannot be decoded by decoders (e.g. decoder doesn't support said image format).
    StaticBytes(Arc<[u8]>)
}