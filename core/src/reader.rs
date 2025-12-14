use std::io::{BufReader, Cursor, Read, Seek};

use crate::image::DecodedImage;

#[derive(Clone, Debug, PartialEq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Svg,
    Gif,
    Webp
}

pub trait ReadSeek: Read + Seek {}
impl<T: Read + Seek> ReadSeek for T {}

pub enum ImageReaderData {
    BufReader(BufReader<Box<dyn ReadSeek>>),
    DecodedImage(DecodedImage),
}

impl<R: ReadSeek + 'static> From<R> for ImageReaderData {
    fn from(reader: R) -> Self {
        Self::BufReader(
            BufReader::new(Box::new(reader))
        )
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