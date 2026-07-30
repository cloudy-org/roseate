use std::io::{Cursor};

use crate::{format::ImageFormat, decoded_image::DecodedImage};

pub type EncodedImageReader = Cursor<Vec<u8>>;

pub enum ImageReaderData {
    EncodedImage(EncodedImageReader),
    DecodedImage(DecodedImage),
}

impl From<EncodedImageReader> for ImageReaderData {
    fn from(cursor: Cursor<Vec<u8>>) -> Self {
        Self::EncodedImage(cursor)
    }
}

impl From<DecodedImage> for ImageReaderData {
    fn from(value: DecodedImage) -> Self {
        Self::DecodedImage(value)
    }
}

pub struct ImageReader {
    // NOTE: this may become private in the future.
    pub(crate) data: ImageReaderData,
    pub image_format: ImageFormat,
}

impl ImageReader {
    pub fn new<T: Into<ImageReaderData>>(data: T, image_format: ImageFormat) -> Self {
        Self {
            data: data.into(),
            image_format
        }
    }
}