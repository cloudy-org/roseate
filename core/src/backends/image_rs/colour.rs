use crate::{error::Error, image::ImageColourType};

impl TryFrom<image::ColorType> for ImageColourType {
    type Error = Error;

    fn try_from(value: image::ColorType) -> Result<Self, Self::Error> {
        let image_colour_type = match value {
            image::ColorType::L8 => Self::Grey8,
            image::ColorType::L16 => Self::Grey16,
            image::ColorType::La8 => Self::GreyA8,
            image::ColorType::La16 => Self::GreyA16,
            image::ColorType::Rgb8 => Self::Rgb8,
            image::ColorType::Rgb16 => Self::Rgb16,
            image::ColorType::Rgb32F => Self::Rgb32F,
            image::ColorType::Rgba8 => Self::Rgba8,
            image::ColorType::Rgba16 => Self::Rgba16,
            image::ColorType::Rgba32F => Self::Rgba32F,
            _ => return Err(Error::UnsupportedColourType)
        };

        Ok(image_colour_type)
    }
}