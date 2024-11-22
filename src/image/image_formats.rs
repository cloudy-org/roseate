use imagesize::ImageType;

use crate::error::Error;

#[derive(Clone, Debug, PartialEq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Svg,
    Gif,
    Webp
}

impl TryFrom<image::ImageFormat> for ImageFormat {
    type Error = Error;

    fn try_from(value: image::ImageFormat) -> Result<Self, Error> {
        let image_format = match value {
            image::ImageFormat::Png => ImageFormat::Png,
            image::ImageFormat::Jpeg => ImageFormat::Jpeg,
            image::ImageFormat::Gif => ImageFormat::Gif,
            image::ImageFormat::WebP => ImageFormat::Webp,
            unsupported_format => {
                return Err(
                    Error::ImageFormatNotSupported(
                        None,
                        format!("{:?}", unsupported_format)
                    )
                );
            },
        };

        Ok(image_format)
    }
}

impl TryFrom<imagesize::ImageType> for ImageFormat {
    type Error = Error;

    // NOTE: Add more formats we know will load later.
    fn try_from(value: imagesize::ImageType) -> Result<Self, Self::Error> {
        let image_format = match value {
            ImageType::Gif => ImageFormat::Gif,
            ImageType::Jpeg => ImageFormat::Jpeg,
            ImageType::Jxl => ImageFormat::Jpeg,
            ImageType::Png => ImageFormat::Png,
            ImageType::Webp => ImageFormat::Webp,
            unsupported_format => {
                return Err(
                    Error::ImageFormatNotSupported(
                        None,
                        format!("{:?}", unsupported_format)
                    )
                );
            },
        };

        Ok(image_format)
    }
}

impl ImageFormat {
    /// Converts roseate's image format to image-rs image format.
    /// 
    /// # Errors
    /// Will return `ImageFormat::PNG` for SVG! Function will not panic though.
    pub fn to_image_rs_format(&self) -> image::ImageFormat {
        match self {
            ImageFormat::Png => image::ImageFormat::Png,
            ImageFormat::Jpeg => image::ImageFormat::Jpeg,
            ImageFormat::Gif => image::ImageFormat::Gif,
            ImageFormat::Webp => image::ImageFormat::WebP,
            ImageFormat::Svg => image::ImageFormat::Png,
        }
    }
}