use std::{collections::HashSet, hash::{DefaultHasher, Hasher}, io::Cursor};

use image::{codecs::{gif::GifEncoder, jpeg::JpegEncoder, png::PngEncoder, webp::WebPEncoder}, DynamicImage, ExtendedColorType, ImageDecoder, ImageEncoder};
use log::debug;

use std::hash::Hash;
use crate::{error::{Error, Result}, image::{fast_downsample::fast_downsample, image_formats::ImageFormat}, notifier::NotifierAPI};

use super::{backends::ImageProcessingBackend, image::{Image, ImageSizeT}};

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

pub enum ModificationProcessingMeat<'a> {
    ImageRS(&'a mut DynamicImage),
    Roseate(&'a mut Vec<u8>, &'a mut ImageSizeT, bool)
}

impl Image {
    pub(super) fn modify_and_decode_image_to_buffer(
        &self,
        image_processing_backend: &ImageProcessingBackend,
        image_decoder: Box<dyn ImageDecoder>,
        modifications: HashSet<ImageModifications>,
        optimized_image_buffer: &mut Vec<u8>,
        notifier: &mut NotifierAPI,
    ) -> Result<()> {
        let image_colour_type = image_decoder.color_type();

        // mutable width and height because some optimizations 
        // modify the image size hence we need to keep track of that.
        let mut actual_image_size = (
            self.image_size.width as u32, self.image_size.height as u32
        );

        match image_processing_backend {
            ImageProcessingBackend::Roseate => {
                let mut pixels = vec![0; image_decoder.total_bytes() as usize];

                debug!("Decoding pixels from image using image decoder...");
                image_decoder.read_image(&mut pixels).unwrap();

                let has_alpha = image_colour_type.has_alpha();

                // TODO: handle result and errors 
                self.apply_modifications(
                    modifications,
                    ModificationProcessingMeat::Roseate(
                        &mut pixels,
                        &mut actual_image_size,
                        has_alpha
                    )
                )?;

                notifier.set_loading(
                    Some("Encoding modified image...".into())
                );
                debug!("Encoding modified image from pixels to a buffer...");

                let (actual_width, actual_height) = actual_image_size;

                let image_result = match self.image_format {
                    ImageFormat::Png => {
                        PngEncoder::new(optimized_image_buffer).write_image(
                            &pixels,
                            actual_width,
                            actual_height,
                            ExtendedColorType::Rgb8
                        )
                    },
                    ImageFormat::Jpeg => {
                        JpegEncoder::new(optimized_image_buffer).write_image(
                            &pixels,
                            actual_width,
                            actual_height,
                            image_colour_type.into()
                        )
                    },
                    ImageFormat::Svg => {
                        PngEncoder::new(optimized_image_buffer).write_image(
                            &pixels,
                            actual_width,
                            actual_height,
                            image_colour_type.into()
                        )
                    },
                    ImageFormat::Gif => {
                        GifEncoder::new(optimized_image_buffer).encode(
                            &pixels,
                            actual_width,
                            actual_height,
                            image_colour_type.into()
                        )
                    },
                    ImageFormat::Webp => {
                        WebPEncoder::new_lossless(optimized_image_buffer).write_image(
                            &pixels,
                            actual_width,
                            actual_height,
                            image_colour_type.into()
                        )
                    },
                };

                match image_result {
                    Ok(_) => Ok(()),
                    Err(error) => Err(
                        Error::ImageFailedToEncode(
                            Some(error.to_string()),
                            "Failed to encode optimized pixels!".to_string()
                        )
                    ),
                }
            },
            ImageProcessingBackend::ImageRS => {
                debug!("Decoding image into dynamic image...");

                let result = DynamicImage::from_decoder(image_decoder);

                match result {
                    Ok(mut dynamic_image) => {
                        // TODO: handle result and errors
                        self.apply_modifications(
                            modifications,
                            ModificationProcessingMeat::ImageRS(&mut dynamic_image)
                        )?;

                        notifier.set_loading(
                            Some("Encoding modified image...".into())
                        );
                        debug!("Encoding modified dynamic image into image buffer...");

                        let image_result = dynamic_image.write_to(
                            &mut Cursor::new(optimized_image_buffer),
                            self.image_format.to_image_rs_format()
                        );

                        match image_result {
                            Ok(_) => Ok(()),
                            Err(error) => Err(
                                Error::ImageFailedToEncode(
                                    Some(error.to_string()),
                                    "Failed to encode optimized dynamic image!".to_string()
                                )
                            ),
                        }
                    },
                    Err(error) => {
                        Err(
                            Error::ImageFailedToDecode(
                                Some(error.to_string()),
                                "Failed to decode image into dynamic image!".to_string()
                            )
                        )
                    }
                }
            }
        }
    }

    fn apply_modifications(
        &self,
        modifications: HashSet<ImageModifications>,
        meat: ModificationProcessingMeat
    ) -> Result<()> {

        match meat {
            ModificationProcessingMeat::ImageRS(dynamic_image) => {

                for modification in modifications {
                    debug!("Applying '{:?}' modification to image...", &modification);

                    if let ImageModifications::Resize((width, height)) = modification {
                        *dynamic_image = dynamic_image.resize(
                            width, height, image::imageops::FilterType::Lanczos3
                        )         
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