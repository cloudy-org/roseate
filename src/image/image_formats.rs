use imagesize::ImageType;

use crate::error::Error;

#[derive(Clone, Debug)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Svg,
    Gif,
    Webp
}

impl ImageFormat {
    // NOTE: Add more formats we know will load later.
    pub fn from_image_size_crate(image_size_image_type: ImageType) -> Result<Self, Error> {
        let image_format = match image_size_image_type {
            ImageType::Gif => ImageFormat::Gif,
            ImageType::Jpeg => ImageFormat::Jpeg,
            ImageType::Jxl => ImageFormat::Jpeg,
            ImageType::Png => ImageFormat::Png,
            ImageType::Webp => ImageFormat::Webp,
            unsupported_format => {
                return Err(
                    Error::ImageFormatNotSupported(
                        format!("{:?}", unsupported_format)
                    )
                );
            },
        };

        Ok(image_format)
    }
}