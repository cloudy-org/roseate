use std::{collections::HashSet, hash::{DefaultHasher, Hasher}};

use cirrus_egui::v1::notifier::Notifier;
use log::debug;
use image::imageops::{resize, FilterType};

use std::hash::Hash;
use crate::{error::{Error, Result}, image::{backends::ModificationProcessingMeat, fast_downsample::fast_downsample}};
use zune_image::image::Image as ZuneImage;

use super::{decode::{DecodedImage, ImageRSImage}, image::{Image, ImageSizeT}};

#[derive(Debug, Clone)]
pub enum ImageModifications {
    Resize(ImageSizeT)    
}

impl Hash for ImageModifications {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            ImageModifications::Resize(_) => "resize".hash(state)
        }
    }
}

impl PartialEq for ImageModifications {
    fn eq(&self, other: &Self) -> bool {
        let mut hasher = DefaultHasher::new();

        if self.hash(&mut hasher) == other.hash(&mut hasher) {
            return true;
        }

        false
    }
}

impl Eq for ImageModifications {}

impl Image {
    /// Will panic if decoded image is of Egui type.
    pub(super) fn modify_decoded_image(
        &self,
        modifications: HashSet<ImageModifications>,
        decoded_image: DecodedImage,
        notifier: &mut Notifier,
    ) -> Result<DecodedImage> {
        notifier.set_loading(Some("Modifying decoded image...".to_string()));

        //let image_colour_type = image_decoder.color_type();

        // mutable width and height because some optimizations 
        // modify the image size hence we need to keep track of that.
        let mut actual_image_size = self.image_size;

        match decoded_image {
            DecodedImage::Egui => unreachable!(),
            DecodedImage::ZuneImage(zune_image) => {
                let mut pixels = zune_image.flatten_to_u8().into_iter().next()
                    .ok_or_else(|| Error::FailedToLoadImage(
                            None,
                            "zune-image backend failed to get image data. This image may be corrupted!".to_string()
                        )
                    )?;

                let colour_space = zune_image.colorspace();
                let has_alpha = colour_space.has_alpha();

                self.apply_modifications(
                    modifications,
                    ModificationProcessingMeat::Roseate(
                        &mut pixels,
                        &mut actual_image_size,
                        has_alpha
                    )
                )?;

                let (actual_width, actual_height) = actual_image_size;

                Ok(
                    DecodedImage::ZuneImage(
                        ZuneImage::from_u8(pixels.as_slice(), actual_width as usize, actual_height as usize, colour_space)
                    )
                )
            },
            DecodedImage::ImageRS(mut image_rs_image) => {
                self.apply_modifications(
                    modifications,
                    ModificationProcessingMeat::ImageRS(&mut image_rs_image)
                )?;

                Ok(DecodedImage::ImageRS(image_rs_image))
            }
        }
    }

    fn apply_modifications(
        &self,
        modifications: HashSet<ImageModifications>,
        meat: ModificationProcessingMeat
    ) -> Result<()> {

        match meat {
            ModificationProcessingMeat::ImageRS(image_rs_image) => {

                for modification in modifications {
                    debug!("Applying '{:?}' modification to image...", &modification);

                    if let ImageModifications::Resize((width, height)) = modification {
                        match image_rs_image {
                            ImageRSImage::RGB(image_buffer) => {
                                *image_buffer = resize(
                                    image_buffer, width, height, FilterType::Lanczos3
                                );
                            },
                            ImageRSImage::RGBA(image_buffer) => {
                                *image_buffer = resize(
                                    image_buffer, width, height, FilterType::Lanczos3
                                );
                            },
                            ImageRSImage::Grey(image_buffer) => {
                                *image_buffer = resize(
                                    image_buffer, width, height, FilterType::Lanczos3
                                );
                            },
                            ImageRSImage::GreyAlpha(image_buffer) => {
                                *image_buffer = resize(
                                    image_buffer, width, height, FilterType::Lanczos3
                                );
                            },
                        }
                    };
                }
            },
            ModificationProcessingMeat::Roseate(pixels, image_size, has_alpha) => {

                for modification in modifications {
                    debug!("Applying '{:?}' modification to image...", &modification);

                    if let ImageModifications::Resize((width, height)) = modification {
                        (*pixels, *image_size) = fast_downsample(
                            pixels,
                            &image_size,
                            (width as u32, height as u32),
                            has_alpha
                        )
                    }
                }
            },
        }

        Ok(())
    }
}