use std::{cmp::min, fmt::Display, io::{BufRead, BufReader, Read}, path::PathBuf};

use imagesize::Compression;

use crate::{
    decoded_image::ImageSize, error::{Error, Result},
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
/// 
/// *It's blazzing fast... 🔥*
pub fn determine_image_format_and_size_from_header<R: Read + ?Sized>(buf_reader: &mut BufReader<Box<R>>) -> Result<(ImageFormat, ImageSize)> {
    // Reads I think 8 KB of the image from disk into ram.
    // 
    // I do this so later the image decoders don't need to read the image header again when we've 
    // already read it here for image size and format. Instead they can just quickly read it from ram 
    // and then read the rest of the image file from disk when it get's past the "image header" window.
    let buffer = buf_reader.fill_buf()
        .map_err(|error| Error::ImageHeaderReadFailure {
            stage: "Failed to read header of image file!".into(),
            error: Some(error.to_string()),
        })?;

    let bytes_to_read_as_header = min(buffer.len(), 1024);

    let image_size_image_type =
        imagesize::image_type(&buffer[..bytes_to_read_as_header]).map_err(|error| {
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

    let image_size = imagesize::blob_size(&buffer[..bytes_to_read_as_header])
        .map_err(|error| Error::ImageHeaderReadFailure {
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
