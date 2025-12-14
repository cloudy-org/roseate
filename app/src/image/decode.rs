use super::{image::{Image, ImageSizeT}, image_data::ImageColourType};

use cirrus_egui::v1::notifier::Notifier;
use log::debug;
use egui_notify::ToastLevel;
use image::{codecs::{jpeg::JpegDecoder, png::PngDecoder, webp::WebPDecoder}, ColorType, ImageDecoder, GrayAlphaImage, GrayImage, RgbImage, RgbaImage};
use zune_image::{codecs::qoi::zune_core::options::DecoderOptions, image::Image as ZuneImage};
use std::{io::{BufReader, Read, Seek}};

use crate::{error::{Error, Result}};

use super::{backends::{ImageDecodePipelineKind, ImageProcessingBackend}, image_formats::ImageFormat};

pub enum ImageRSImage {
    RGB(RgbImage),
    RGBA(RgbaImage),
    /// Luma
    Grey(GrayImage),
    /// LumaA
    GreyAlpha(GrayAlphaImage),
}

pub enum DecodedImage {
    ImageRS(ImageRSImage),
    ZuneImage(ZuneImage),
    /// Let egui decode the image.
    Egui
}

impl Image {
    pub(super) fn decode_image<'a, R: Read + Seek + 'a>(
        &self,
        image_processing_backend: &ImageProcessingBackend,
        image_buf_reader: &'a mut BufReader<R>,
        notifier: &mut Notifier
    ) -> Result<DecodedImage> {
        match image_processing_backend.get_decode_pipeline() {
            ImageDecodePipelineKind::ImageRS => {
                let image_decoder: Box<dyn ImageDecoder + 'a> = match self.image_format {
                    ImageFormat::Png => Box::new(PngDecoder::new(image_buf_reader).unwrap()),
                    ImageFormat::Jpeg => Box::new(JpegDecoder::new(image_buf_reader).unwrap()),
                    ImageFormat::Svg => return Ok(DecodedImage::Egui),
                    ImageFormat::Gif => return Ok(DecodedImage::Egui),
                    ImageFormat::Webp => Box::new(WebPDecoder::new(image_buf_reader).unwrap()),
                };

                let (width, height) = image_decoder.dimensions();
                let image_colour_type = image_decoder.color_type();

                debug!("Decoding image using image-rs decoder...");

                let mut image_buffer = vec![0; image_decoder.total_bytes() as usize];
                if let Err(error) = image_decoder.read_image(&mut image_buffer) {
                    return Err(
                        Error::FailedToDecodeImage(
                            Some(error.to_string()),
                            "We failed to read this image. It seems bigger than we \
                            anticipated. This is most likely a bug, report it.".to_string()
                        )
                    )
                }

                // NOTE: roseate won't support 8-bit+ (or HDR) images for now, will look into it in the future 
                let image_rs_image = match image_colour_type {
                    ColorType::L8 => ImageRSImage::Grey(GrayImage::from_raw(width, height, image_buffer).unwrap()),
                    ColorType::La8 => ImageRSImage::GreyAlpha(GrayAlphaImage::from_raw(width, height, image_buffer).unwrap()),
                    ColorType::Rgb8 => ImageRSImage::RGB(RgbImage::from_raw(width, height, image_buffer).unwrap()),
                    ColorType::Rgba8 => ImageRSImage::RGBA(RgbaImage::from_raw(width, height, image_buffer).unwrap()),
                    ColorType::L16 => ImageRSImage::Grey(GrayImage::from_raw(width, height, image_buffer).unwrap()),
                    ColorType::La16 => ImageRSImage::GreyAlpha(GrayAlphaImage::from_raw(width, height, image_buffer).unwrap()),
                    ColorType::Rgb16 => ImageRSImage::RGB(RgbImage::from_raw(width, height, image_buffer).unwrap()),
                    ColorType::Rgba16 => ImageRSImage::RGBA(RgbaImage::from_raw(width, height, image_buffer).unwrap()),
                    ColorType::Rgb32F => ImageRSImage::RGB(RgbImage::from_raw(width, height, image_buffer).unwrap()),
                    ColorType::Rgba32F => ImageRSImage::RGBA(RgbaImage::from_raw(width, height, image_buffer).unwrap()),
                    _ => ImageRSImage::RGBA(
                        RgbaImage::from_raw(width, height, image_buffer).ok_or_else(
                            || Error::FailedToDecodeImage(
                                None,
                                "image-rs backend failed to get colour space of image \
                                so we had to guess, but it the guess was incorrect! Image is maybe corrupted!".to_string()
                            )
                        )?
                    ),
                };

                Ok(DecodedImage::ImageRS(image_rs_image))
            },
            ImageDecodePipelineKind::ZuneImage => {
                let result = match &self.image_format {
                    // ZumeImage at the moment only supports decoding from our png and jpeg formats.
                    ImageFormat::Png | ImageFormat::Jpeg => {
                        debug!("Decoding image using zune-image decoder...");

                        let mut buffer = Vec::new();
                        // TODO: handle error
                        image_buf_reader.read_to_end(&mut buffer).unwrap();

                        ZuneImage::read(buffer, DecoderOptions::new_fast())
                    },
                    unsupported_image_format => {
                        notifier.toast(
                            format!(
                                "The zune-image backend does not support decoding the image \
                                    format '{:#}' so we've fallen back to something that works.",
                                unsupported_image_format
                            ),
                            ToastLevel::Warning,
                            |_| {}
                        );

                        return self.decode_image(&ImageProcessingBackend::ImageRS, image_buf_reader, notifier);
                    }
                };

                match result {
                    Ok(zune_image) => Ok(
                        DecodedImage::ZuneImage(zune_image)
                    ),
                    Err(error) => {
                        Err(
                            Error::FailedToDecodeImage(
                                Some(error.to_string()),
                                "Failed to decode image using zune-image backend!".to_string()
                            )
                        )
                    }
                }
            }
        }
    }

    pub(super) fn decoded_image_to_pixels(&self, decoded_image: DecodedImage) -> Result<(Vec<u8>, ImageSizeT, ImageColourType)> {
        match decoded_image {
            DecodedImage::Egui => unreachable!(),
            DecodedImage::ZuneImage(zune_image) => {
                let pixels = zune_image.flatten_to_u8().into_iter().next()
                    .ok_or_else(|| Error::FailedToConvertImageToPixels(
                            None,
                            "zune-image backend failed to get image data. This image may be corrupted!".to_string()
                        )
                    )?;

                let dimensions = zune_image.dimensions();

                Ok((pixels, (dimensions.0 as u32, dimensions.1 as u32), zune_image.colorspace().try_into()?))
            },
            DecodedImage::ImageRS(image_rs_image) => {
                match image_rs_image {
                    ImageRSImage::RGB(image_buffer) => {
                        let dimensions = image_buffer.dimensions();
                        Ok((image_buffer.into_raw(), (dimensions.0, dimensions.1), ImageColourType::RGB))
                    },
                    ImageRSImage::RGBA(image_buffer) => {
                        let dimensions = image_buffer.dimensions();
                        Ok((image_buffer.into_raw(), (dimensions.0, dimensions.1), ImageColourType::RGBA))
                    },
                    ImageRSImage::Grey(image_buffer) => {
                        let dimensions = image_buffer.dimensions();
                        Ok((image_buffer.into_raw(), (dimensions.0, dimensions.1), ImageColourType::Grey))
                    },
                    ImageRSImage::GreyAlpha(image_buffer) => {
                        let dimensions = image_buffer.dimensions();
                        Ok((image_buffer.into_raw(), (dimensions.0, dimensions.1), ImageColourType::GreyAlpha))
                    },
                }
            }
        }
    }
}