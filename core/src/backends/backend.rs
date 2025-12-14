use crate::{decoded_image::DecodedImage, error::Result, modifications::ImageModification, reader::ImageReader};

pub trait DecodeBackend {
    fn from_reader(image_reader: ImageReader) -> Result<Self> where Self: Sized;
    fn modify(&mut self, modifications: Vec<ImageModification>);
    // We use "self" instead of "&mut self", as decode will always be the final function call on this struct.
    // After this function call and once we've receive "DecodedImage" we no longer need this struct any more.
    fn decode(self) -> Result<DecodedImage>;
}