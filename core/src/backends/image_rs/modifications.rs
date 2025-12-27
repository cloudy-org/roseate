use std::collections::HashSet;

use image::imageops::{self, FilterType};

use crate::{backends::image_rs::{ImageRSBackend, buffer_image::{BufferImage, BufferImageVariant}}, modifications::ImageModification};

impl ImageRSBackend {

    pub(super) fn apply_modifications_to_buffer_image(modifications: HashSet<ImageModification>, buffer_image: &mut BufferImage) {
        // cloning shouldn't be too expensive, if that changes in the future we adjust this
        for modification in modifications {

            match modification {
                ImageModification::Resize(width, height) => {
                    log::debug!("Applying resize modification ({}x{})...", width, height);

                    let variant = &mut buffer_image.variant;

                    *variant = match &variant {
                        BufferImageVariant::Grey8(image_buffer) => {
                            BufferImageVariant::Grey8(
                                imageops::resize(
                                    image_buffer, width, height, FilterType::Lanczos3
                                )
                            )
                        },
                        BufferImageVariant::GreyA8(image_buffer) => {
                            BufferImageVariant::GreyA8(
                                imageops::resize(
                                    image_buffer, width, height, FilterType::Lanczos3
                                )
                            )
                        },
                        BufferImageVariant::Rgb8(image_buffer) => {
                            BufferImageVariant::Rgb8(
                                imageops::resize(
                                    image_buffer, width, height, FilterType::Lanczos3
                                )
                            )
                        },
                        BufferImageVariant::Rgba8(image_buffer) => {
                            BufferImageVariant::Rgba8(
                                imageops::resize(
                                    image_buffer, width, height, FilterType::Lanczos3
                                )
                            )
                        },
                    };
                },
            }

        }
    }
}