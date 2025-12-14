use std::{fs, path::Path};

use image::{ImageBuffer, Rgba};
use roseate_core::{format::ImageFormat, image::{DecodedImage, DecodedImageContent}};

mod test_image_rs_backend;

pub const IMAGE_DUMP_PATH: &str = "./tests-image-dump";

pub fn save_image_as_rgba(decoded_image: DecodedImage, name: &str) {
    let _ = fs::create_dir(IMAGE_DUMP_PATH);

    let (width, height) = decoded_image.size;

    match decoded_image.content {
        DecodedImageContent::Static(pixels) => {
            let image: ImageBuffer<Rgba<u8>, _> = ImageBuffer::from_raw(
                decoded_image.size.0, decoded_image.size.1, pixels
            ).unwrap();

            image.save(Path::new(IMAGE_DUMP_PATH).join(name)).unwrap();
        },
        DecodedImageContent::Animated(frames) => {
            for (index, (frame_pixels, _)) in frames.into_iter().enumerate() {
                let frame_image: ImageBuffer<Rgba<u8>, _> = ImageBuffer::from_raw(
                    width, height, frame_pixels
                ).unwrap();

                let (file_name, prefix) = name.split_once(".")
                    .unwrap_or((
                        name,
                        match decoded_image.image_format {
                            ImageFormat::Png => "png",
                            ImageFormat::Jpeg => "jpeg",
                            ImageFormat::Svg => "svg",
                            ImageFormat::Gif => "gif",
                            ImageFormat::Webp => "wedp",
                        }
                    ));

                let image_path = Path::new(IMAGE_DUMP_PATH)
                    .join(file_name);

                let _ = fs::create_dir(&image_path);

                frame_image.save(
                    image_path.join(format!("frame_{}.{}", index, prefix))
                ).unwrap();
            }
        }
    }
}