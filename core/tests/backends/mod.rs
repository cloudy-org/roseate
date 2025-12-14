use std::path::Path;

use image::{ImageBuffer, Rgba};
use roseate_core::image::{DecodedImage, DecodedImageContent};

mod test_image_rs_backend;

pub const IMAGE_DUMP_PATH: &str = "./tests-image-dump";

pub fn save_image(decoded_image: DecodedImage, name: &str) {
    if let DecodedImageContent::Static(pixels) = decoded_image.content {
        let image: ImageBuffer<Rgba<u8>, _> = ImageBuffer::from_raw(
            decoded_image.size.0, decoded_image.size.1, pixels
        ).unwrap();

        image.save(Path::new(IMAGE_DUMP_PATH).join(name)).unwrap();
    }
}