use log::debug;
use roseate_core::{decoded_image::{DecodedImageContent, ImageSize}, fast_downsample::experimental_fast_downsample, modifications::{ImageModification, ImageModifications}};

use crate::{image::Image, image_handler::ImageHandler};

impl ImageHandler {
    pub fn snatch_resize_modification_and_get_size(image_modifications: &mut ImageModifications) -> Option<ImageSize> {
        for modification in image_modifications.clone().iter() {
            #[allow(irrefutable_let_patterns)]
            if let ImageModification::Resize(width, height) = modification {
                image_modifications.remove(&modification);
                return Some((*width, *height));
            }
        }

        None
    }

    pub fn perform_multi_threaded_downsample(target_size: ImageSize, image: &mut Image, number_of_threads: Option<usize>) {
        debug!("Using the experimental multi-threaded function to downsample this image...");

        if let Some(decoded_image) = image.decoded.lock().unwrap().as_mut() {
            match &mut decoded_image.content {
                DecodedImageContent::Static(pixels) => {
                    (*pixels, decoded_image.size) = experimental_fast_downsample(
                        pixels,
                        target_size,
                        &decoded_image.size,
                        &decoded_image.colour_type,
                        number_of_threads
                    );
                },
                DecodedImageContent::Animated(frames) => {
                    let mut index = 0;

                    for (pixels, _) in frames {
                        debug!("Downsampling frame {}...", index);

                        // This will need testing.
                        (*pixels, decoded_image.size) = experimental_fast_downsample(
                            pixels,
                            target_size,
                            &decoded_image.size,
                            &decoded_image.colour_type,
                            number_of_threads
                        );

                        index += 1;
                    }
                },
            }
        }
    }
}