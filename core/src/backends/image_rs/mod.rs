use std::{collections::HashSet, time::Duration};

use image::{AnimationDecoder, ImageDecoder, codecs::{gif::GifDecoder, jpeg::JpegDecoder, png::PngDecoder, webp::WebPDecoder}, imageops::{self, FilterType}};

use crate::{backends::{backend::{DecodeBackend, ModificationBackend}, image_rs::buffer_image::BufferImage}, error::{Error, Result}, image::{DecodedImage, DecodedImageContent, ImageColourType}, modifications::ImageModification, reader::{ImageFormat, ImageReader, ImageReaderData}};

mod buffer_image;

enum Decoder {
    Image(Box<dyn ImageDecoder>),
    Animation(Box<dyn for<'a> AnimationDecoder<'a>>),
}

enum Buffer {
    Image(BufferImage),
    Animation((Vec<BufferImage>, Duration))
}

enum Source {
    Decoder(Decoder),
    Buffer(Buffer),
}

pub struct ImageRSBackend {
    source: Source,
    modifications: HashSet<ImageModification>,
    image_format: ImageFormat,
}

impl ModificationBackend for ImageRSBackend {
    fn modify(&mut self, modifications: Vec<ImageModification>) {
        self.modifications.extend(modifications);
    }
}

impl DecodeBackend for ImageRSBackend {
    // TODO: look into potentially changing that "'static" reference 
    fn from_reader(image_reader: ImageReader) -> Result<Self> {
        match image_reader.data {
            ImageReaderData::BufReader(buf_reader) => {
                log::debug!("Initializing image-rs backend decoders with buf reader...");

                if let ImageFormat::Gif = image_reader.image_format {
                    return Ok(
                        Self {
                            // TODO: handle result
                            source: Source::Decoder(
                                Decoder::Animation(Box::new(GifDecoder::new(buf_reader).unwrap()))
                            ),
                            modifications: HashSet::new(),
                            image_format: image_reader.image_format
                        }
                    )
                }

                // TODO: Don't unwrap and handle image-rs's error 
                // properly by mapping it correctly to roseate-core's Error enum.
                let image_decoder: Box<dyn ImageDecoder> = match image_reader.image_format {
                    ImageFormat::Png => Box::new(PngDecoder::new(buf_reader).unwrap()),
                    ImageFormat::Jpeg => Box::new(JpegDecoder::new(buf_reader).unwrap()),
                    ImageFormat::Webp => Box::new(WebPDecoder::new(buf_reader).unwrap()),
                    _ => return Err(Error::UnsupportedImageFormat)
                };

                Ok(
                    Self {
                        source: Source::Decoder(Decoder::Image(image_decoder)),
                        modifications: HashSet::new(),
                        image_format: image_reader.image_format
                    }
                )
            },
            ImageReaderData::DecodedImage(decoded_image) => {
                log::debug!("Initializing image-rs backend from decoded image...");

                match decoded_image.content {
                    DecodedImageContent::Static(pixels) => {
                        // TODO: Handle results
                        // If we're coming from a decoded image it won't support any other pixel variants other than u8.
                        let image_buffer = BufferImage::from_u8_pixels(
                            pixels,
                            decoded_image.size,
                            decoded_image.colour_type
                        )?;

                        Ok(
                            Self {
                                source: Source::Buffer(Buffer::Image(image_buffer)),
                                modifications: HashSet::new(),
                                image_format: image_reader.image_format
                            }
                        )
                    },
                    DecodedImageContent::Animated(pixels) => todo!(),
                }
            },
        }
    }

    fn decode(self) -> Result<DecodedImage> {
        match self.source {
            Source::Decoder(decoder) => {
                match decoder {
                    Decoder::Image(image_decoder) => {
                        log::debug!("Decoding image with image-rs decoder...");

                        let mut image_pixels: Vec<u8> = vec![0; image_decoder.total_bytes() as usize];

                        let image_size = image_decoder.dimensions();
                        let image_colour_type = match image_decoder.color_type() {
                            image::ColorType::L8 => ImageColourType::Grey8,
                            image::ColorType::L16 => ImageColourType::Grey16,
                            image::ColorType::La8 => ImageColourType::GreyA8,
                            image::ColorType::La16 => ImageColourType::GreyA16,
                            image::ColorType::Rgb8 => ImageColourType::Rgb8,
                            image::ColorType::Rgb16 => ImageColourType::Rgb16,
                            image::ColorType::Rgb32F => ImageColourType::Rgb32F,
                            image::ColorType::Rgba8 => ImageColourType::Rgba8,
                            image::ColorType::Rgba16 => ImageColourType::Rgba16,
                            image::ColorType::Rgba32F => ImageColourType::Rgba32F,
                            _ => return Err(Error::UnsupportedColourType)
                        };

                        if let Err(error) = image_decoder.read_image(&mut image_pixels) {
                            // TODO: map extract image rs error to a roseate-core error.
                            return Err(Error::DecodingError);
                        }

                        log::debug!("Image-rs decoder successfully decoded to pixels...");

                        if self.modifications.is_empty() {
                            log::debug!(
                                "No image modifications so we're constructing decoded image directly from image pixels..."
                            );

                            return Ok(
                                DecodedImage::new(
                                    DecodedImageContent::Static(image_pixels),
                                    image_colour_type,
                                    self.image_format,
                                    image_size,
                                )
                            );
                        }

                        log::debug!(
                            "We have image modifications. Constructing image-rs image buffer to apply modifications..."
                        );

                        let mut buffer_image = BufferImage::from_u8_pixels(
                            image_pixels, image_size, image_colour_type
                        )?;

                        log::debug!("Image buffer constructed, applying modifications...");

                        Self::apply_modifications_to_buffer_image(self.modifications, &mut buffer_image);

                        log::debug!("Converting image buffer back to pixels to construct into decoded image...");

                        let (image_size, image_pixels, image_colour_type) = buffer_image.to_u8_pixels();

                        Ok(
                            DecodedImage::new(
                                DecodedImageContent::Static(image_pixels),
                                image_colour_type,
                                self.image_format,
                                image_size,
                            )
                        )
                    },
                    Decoder::Animation(animation_decoder) => todo!(),
                }
            },
            Source::Buffer(buffer) => {
                match buffer {
                    Buffer::Image(mut buffer_image) => {
                        log::debug!("Image already decoded and constructed as image-rs image buffer, applying modifications...");

                        Self::apply_modifications_to_buffer_image(self.modifications, &mut buffer_image);

                        let (size, pixels, colour_type) = buffer_image.to_u8_pixels();

                        Ok(
                            DecodedImage::new(
                                DecodedImageContent::Static(pixels),
                                colour_type,
                                self.image_format,
                                size
                            )
                        )
                    },
                    Buffer::Animation(_) => todo!(),
                }
            },
        }
    }
}

impl ImageRSBackend {
    fn apply_modifications_to_buffer_image(modifications: HashSet<ImageModification>, buffer_image: &mut BufferImage) {
        // cloning shouldn't be too expensive, if that changes in the future we adjust this
        for modification in modifications {

            match modification {
                ImageModification::Resize(width, height) => {
                    log::debug!("Applying resize modification ({}x{})...", width, height);

                    *buffer_image = match buffer_image {
                        BufferImage::Grey8(image_buffer) => {
                            BufferImage::Grey8(
                                imageops::resize(
                                    image_buffer, width, height, FilterType::Lanczos3
                                )
                            )
                        },
                        BufferImage::GreyA8(image_buffer) => {
                            BufferImage::GreyA8(
                                imageops::resize(
                                    image_buffer, width, height, FilterType::Lanczos3
                                )
                            )
                        },
                        BufferImage::Rgb8(image_buffer) => {
                            BufferImage::Rgb8(
                                imageops::resize(
                                    image_buffer, width, height, FilterType::Lanczos3
                                )
                            )
                        },
                        BufferImage::Rgba8(image_buffer) => {
                            BufferImage::Rgba8(
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