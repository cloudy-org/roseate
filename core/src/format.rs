use std::{fmt::Display, io::{Seek, SeekFrom}, path::PathBuf};

use imagesize::Compression;

use crate::{
    decoded_image::ImageSize, error::{Error, Result}, reader::EncodedImageReader,
};

pub const IMAGE_FORMAT_EXTENSIONS: &[&str] = &[
    "png",
    "jpg", "jpeg",
    // "svg",
    "gif", "gifv",
    "webp",
    "avif",
    "tiff", "tif",
    "qoi",
    "bmp",
    "ico",
];

#[derive(Clone, Debug, PartialEq, Hash)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Svg,
    Gif,
    Webp,
    Avif,
    Tiff,
    Qoi,
    Bmp,
    Ico
}

impl Display for ImageFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageFormat::Png => write!(f, "PNG (Portable Network Graphics)"),
            ImageFormat::Jpeg => write!(f, "JPEG (Joint Photographic Experts Group)"),
            ImageFormat::Svg => write!(f, "SVG (Scalable Vector Graphics)"),
            ImageFormat::Gif => write!(f, "GIF (Graphics Interchange Format)"),
            ImageFormat::Webp => write!(f, "WEBP (Web Picture)"),
            ImageFormat::Avif => write!(f, "AVIF (AV1 Image File Format)"),
            ImageFormat::Tiff => write!(f, "TIFF (Tagged Image File Format)"),
            ImageFormat::Qoi => write!(f, "QOI (Quite OK Image Format)"),
            ImageFormat::Bmp => write!(f, "BMP (Bitmap)"),
            ImageFormat::Ico => write!(f, "ICO (Microsoft Icon)"),
        }
    }
}

/// Only reads the header of an image and determines it's image format and size from that.
pub fn determine_image_format_and_size_from_header(encoded_image_reader: &mut EncodedImageReader) -> Result<(ImageFormat, ImageSize)> {
    let image_size_image_type = imagesize::reader_type(&mut *encoded_image_reader)
        .map_err(|error| {
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
        imagesize::ImageType::Heif(Compression::Av1) => ImageFormat::Avif,
        imagesize::ImageType::Tiff => ImageFormat::Tiff,
        imagesize::ImageType::Qoi => ImageFormat::Qoi,
        imagesize::ImageType::Bmp => ImageFormat::Bmp,
        imagesize::ImageType::Ico => ImageFormat::Ico,
        unsupported_format => {
            return Err(
                Error::ImageFormatNotSupported {
                    image_format: format!("{:?}", unsupported_format),
                }
            );
        }
    };

    // TODO: if this fails with a "failed to fill whole buffer" retry again with the whole buf reader.
    // Some more JPEGs are failing and most TIF also are failing. 
    let image_size = image_size_image_type.reader_size(&mut *encoded_image_reader)
        .map_err(|error| Error::ImageHeaderReadFailure {
            stage: "Failed to retrieve image dimensions!".into(),
            error: Some(error.to_string()),
        })?;

    encoded_image_reader.seek(SeekFrom::Start(0))
        .map_err(|error| {
            Error::ImageHeaderReadFailure {
                stage: "Failed to seek back to start after image format and size read.".into(),
                error: Some(error.to_string()),
            }
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
