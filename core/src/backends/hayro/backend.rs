use crate::{backends::backend::DecodeBackend, error::Result, format::ImageFormat, reader::ImageReader};

pub struct Hayro {

}

impl DecodeBackend for Hayro {
    const SUPPORTED_FORMATS: &[ImageFormat] = &[
        ImageFormat::Pdf
    ];

    fn from_reader(image_reader: ImageReader) -> Result<Self> {
        todo!()
    }

    fn modify<I>(&mut self, modifications: I)
    where
        I: IntoIterator<Item = crate::modifications::ImageModification> {
        todo!()
    }

    fn decode(self) -> crate::error::Result<crate::decoded_image::DecodedImage> {
        todo!()
    }
}