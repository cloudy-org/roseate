use std::{fmt::Display, fs::File, io::Read, path::PathBuf};

use crate::{
    decoded_image::ImageSize,
    error::{Error, Result},
};

#[derive(Clone, Debug, PartialEq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Svg,
    Gif,
    Webp,
    Tiff,
}

impl Display for ImageFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageFormat::Png => write!(f, "PNG (Portable Network Graphics)"),
            ImageFormat::Jpeg => write!(f, "JPEG (Joint Photographic Experts Group)"),
            ImageFormat::Svg => write!(f, "SVG (Scalable Vector Graphics)"),
            ImageFormat::Gif => write!(f, "GIF (Graphics Interchange Format)"),
            ImageFormat::Webp => write!(f, "WEBP (Web Picture)"),
            ImageFormat::Tiff => write!(f, "TIFF (Tagged Image File Format)"),
        }
    }
}

/// Only reads the header of an image and determines it's image format and size from that.
///
/// *It's blazzing fast... 🔥*
pub fn determine_image_format_and_size_from_header(
    path: &PathBuf,
) -> Result<(ImageFormat, ImageSize)> {
    // TODO: figure out how we can share the same buf reader used
    // for image decoding to improve speed and save on I/O calls.
    let mut buffer = [0u8; 1024];
    let number_of_bytes_read = File::open(path)
        .map_err(|error| Error::ImageHeaderReadFailure {
            stage: "Failed to open file!".into(),
            error: Some(error.to_string()),
        })?
        .read(&mut buffer)
        .map_err(|error| Error::ImageHeaderReadFailure {
            stage: "Failed to read header of image file!".into(),
            error: Some(error.to_string()),
        })?;

    let image_size_image_type =
        imagesize::image_type(&buffer[..number_of_bytes_read]).map_err(|error| {
            Error::ImageHeaderReadFailure {
                stage: "Failed to determine format of image!".into(),
                error: Some(error.to_string()),
            }
        })?;

    let image_format = match image_size_image_type {
        imagesize::ImageType::Gif => ImageFormat::Gif,
        imagesize::ImageType::Jpeg => ImageFormat::Jpeg,
        imagesize::ImageType::Png => ImageFormat::Png,
        imagesize::ImageType::Webp => ImageFormat::Webp,
        imagesize::ImageType::Tiff => ImageFormat::Tiff,
        unsupported_format => {
            return Err(Error::ImageFormatNotSupported {
                image_format: format!("{:?}", unsupported_format),
            });
        }
    };

    // TODO: when we switch to shared buf reader we should stop using path
    let image_size = imagesize::size(path).map_err(|error| Error::ImageHeaderReadFailure {
        stage: "Failed to retrieve image dimensions!".into(),
        error: Some(error.to_string()),
    })?;

    Ok((
        image_format,
        (image_size.width as u32, image_size.height as u32),
    ))
}

pub fn determine_svg_size(path: &PathBuf) -> ImageSize {
    let metadata = svg_metadata::Metadata::parse_file(&path)
        .expect("Failed to parse metadata of the svg file!");

    let width = metadata.width().expect("Failed to get SVG width!");
    let height = metadata.height().expect("Failed to get SVG height!");

    (width as u32, height as u32)
}
