use std::{collections::HashSet, io::BufReader};

use image::{AnimationDecoder, ImageDecoder, ImageError, codecs::{gif::GifDecoder, jpeg::JpegDecoder, png::PngDecoder, webp::WebPDecoder}, imageops::{self, FilterType}};
use log::debug;

use crate::{backends::{backend::DecodeBackend, image_rs::buffer_image::BufferImage}, colour_type::ImageColourType, decoded_image::{DecodedImage, DecodedImageContent, ImageSize, Pixels}, error::{Error, Result}, format::ImageFormat, modifications::{self, ImageModification, ImageModifications}, reader::{ImageReader, ImageReaderData, ReadSeek}};

mod colour;
mod buffer_image;

// TODO: Fill with debug logs

enum Decoder {
    Png(PngDecoder<BufReader<Box<dyn ReadSeek>>>),
    Jpeg(JpegDecoder<BufReader<Box<dyn ReadSeek>>>),
    Webp(WebPDecoder<BufReader<Box<dyn ReadSeek>>>),
    Gif(GifDecoder<BufReader<Box<dyn ReadSeek>>>),
}

enum Buffer {
    Image(BufferImage),
    Animation((Vec<(BufferImage, f32)>, ImageSize, ImageColourType))
}

enum Source {
    Decoder(Decoder),
    Buffer(Buffer),
}

pub struct ImageRSBackend {
    source: Source,
    modifications: ImageModifications,
    image_format: ImageFormat,
}

impl DecodeBackend for ImageRSBackend {
    fn from_reader(image_reader: ImageReader) -> Result<Self> {
        match image_reader.data {
            ImageReaderData::BufReader(buf_reader) => {
                log::debug!("Initializing image-rs backend decoders with buf reader...");

                let error_func = |error: ImageError| { 
                    Error::DecoderInitFailure { error: error.to_string() }
                };

                // TODO: Don't unwrap and handle image-rs's error 
                // properly by mapping it correctly to roseate-core's Error enum.
                let image_decoder = match image_reader.image_format {
                    ImageFormat::Gif => Decoder::Gif(GifDecoder::new(buf_reader).map_err(error_func)?),
                    ImageFormat::Png => Decoder::Png(PngDecoder::new(buf_reader).map_err(error_func)?),
                    ImageFormat::Jpeg => Decoder::Jpeg(JpegDecoder::new(buf_reader).map_err(error_func)?),
                    ImageFormat::Webp => Decoder::Webp(WebPDecoder::new(buf_reader).map_err(error_func)?),
                    unsupported_format => return Err(
                        Error::DecoderNotSupported {
                            image_format: unsupported_format.to_string(),
                            backend: String::from("image-rs")
                        }
                    )
                };

                Ok(
                    Self {
                        source: Source::Decoder(image_decoder),
                        modifications: HashSet::new(),
                        image_format: image_reader.image_format
                    }
                )
            },
            ImageReaderData::DecodedImage(decoded_image) => {
                log::debug!("Initializing image-rs backend from already decoded image...");

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
                    DecodedImageContent::Animated(frames) => {
                        let mut animated_buffers = Vec::new();

                        for (pixels, delay) in frames {
                            let image_buffer = BufferImage::from_u8_pixels(
                                pixels,
                                decoded_image.size,
                                decoded_image.colour_type
                            )?;

                            animated_buffers.push((image_buffer, delay));
                        }

                        Ok(
                            Self {
                                source: Source::Buffer(
                                    Buffer::Animation(
                                        (animated_buffers, decoded_image.size, decoded_image.colour_type)
                                    )
                                ),
                                modifications: HashSet::new(),
                                image_format: image_reader.image_format
                            }
                        )
                    },
                }
            },
        }
    }

    fn modify<I>(&mut self, modifications: I)
    where
        I: IntoIterator<Item = ImageModification>
    {
        self.modifications.extend(modifications);
    }

    fn decode(self) -> Result<DecodedImage> {
        match self.source {
            Source::Decoder(decoder) => {
                match decoder {
                    Decoder::Png(png_decoder) => {
                        let has_animation = png_decoder.is_apng().map_err(|error| Error::AnimationCheckError(
                            format!(
                                "The upstream image-rs png decoder unexpectedly failed to check if the image was animated: {}",
                                error.to_string()
                            )
                        ))?;

                        match has_animation {
                            true => {
                                let apng_decoder = png_decoder.apng()
                                    .expect("We should have been given the image-rs APNG Decoder but we weren't!");

                                Self::decode_animated_image(apng_decoder, self.modifications, self.image_format)
                            },
                            false => Self::decode_image(png_decoder, self.modifications, self.image_format)
                        }
                    },
                    Decoder::Webp(webp_decoder) => {
                        match webp_decoder.has_animation() {
                            true => Self::decode_animated_image(webp_decoder, self.modifications, self.image_format),
                            false => Self::decode_image(webp_decoder, self.modifications, self.image_format),
                        }
                    },
                    Decoder::Gif(gif_decoder) => Self::decode_animated_image(gif_decoder, self.modifications, self.image_format),
                    Decoder::Jpeg(jpeg_decoder) => Self::decode_image(jpeg_decoder, self.modifications, self.image_format),
                }
            },
            Source::Buffer(buffer) => {
                log::debug!("Image already decoded and constructed as image-rs image buffer, applying modifications...");

                match buffer {
                    Buffer::Image(mut buffer_image) => {
                        Self::apply_modifications_to_buffer_image(self.modifications, &mut buffer_image);

                        let (pixels, size, colour_type) = buffer_image.to_u8_pixels();

                        Ok(
                            DecodedImage::new(
                                DecodedImageContent::Static(pixels),
                                colour_type,
                                self.image_format,
                                size
                            )
                        )
                    },
                    Buffer::Animation((buffer_image_frames, image_size, image_colour_type)) => {
                        let mut animated_pixels = Vec::new();

                        for (index, (mut buffer_image, delay)) in buffer_image_frames.into_iter().enumerate() {
                            debug!("Applying modifications to frame {}...", index);

                            Self::apply_modifications_to_buffer_image(self.modifications.clone(), &mut buffer_image);

                            let (pixels, _, _) = buffer_image.to_u8_pixels();

                            animated_pixels.push((pixels, delay));
                        }

                        Ok(
                            DecodedImage::new(
                                DecodedImageContent::Animated(animated_pixels),
                                image_colour_type,
                                self.image_format,
                                image_size
                            )
                        )
                    },
                }
            }
        }
    }
}

impl ImageRSBackend {
    fn decode_animated_image<'a, T: AnimationDecoder<'a>>(
        animation_decoder: T,
        modifications: ImageModifications,
        image_format: ImageFormat
    ) -> Result<DecodedImage> {
        let mut image_size: Option<ImageSize> = None;

        let mut image_pixels: Vec<(Pixels, f32)> = Vec::new();

        let perform_modifications = !modifications.is_empty();

        for frame_result in animation_decoder.into_frames() {
            let frame = frame_result.map_err(
                // NOTE: I might change this to a less generic error.
                |error| Error::DecodingFailure {
                    error: format!("Image-rs decoder failed to decode animated frame: {}", error.to_string())
                }
            )?;

            let (numerator, denominator) = frame.delay().numer_denom_ms();
            let delay_seconds = (numerator as f32 / denominator as f32) / 1000.0;

            let image_buffer = frame.into_buffer();

            let mut buffer_image = BufferImage::Rgba8(image_buffer);

            if perform_modifications {
                Self::apply_modifications_to_buffer_image(modifications.clone(), &mut buffer_image);
            }

            let (pixels, size, _) = buffer_image.to_u8_pixels();

            image_size.get_or_insert(size);

            image_pixels.push((pixels, delay_seconds));
        }

        Ok(
            DecodedImage::new(
                DecodedImageContent::Animated(image_pixels),
                ImageColourType::Rgba8,
                image_format,
                // TODO: don't unwrap and find a better way to handle this
                image_size.expect("We should have image size unless there was zero animated frames!"),
            )
        )
    }

    fn decode_image<T: ImageDecoder>(image_decoder: T, modifications: ImageModifications, image_format: ImageFormat) -> Result<DecodedImage> {
        log::debug!("Decoding image with image-rs decoder...");

        let mut image_pixels: Vec<u8> = vec![0; image_decoder.total_bytes() as usize];

        let image_size = image_decoder.dimensions();
        let image_colour_type = ImageColourType::try_from(image_decoder.color_type())?;

        if let Err(error) = image_decoder.read_image(&mut image_pixels) {
            // TODO: map extract image rs error to a roseate-core error.
            return Err(
                // NOTE: I might change this to a less generic error.
                Error::DecodingFailure {
                    error: format!("Image-rs decoder failed to decode image to pixels: {}", error.to_string()),
                }
            );
        }

        log::debug!("Image-rs decoder successfully decoded to pixels...");

        if modifications.is_empty() {
            log::debug!(
                "No image modifications so we're constructing decoded image directly from image pixels..."
            );

            return Ok(
                DecodedImage::new(
                    DecodedImageContent::Static(image_pixels),
                    image_colour_type,
                    image_format,
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

        Self::apply_modifications_to_buffer_image(modifications, &mut buffer_image);

        log::debug!("Converting image buffer back to pixels to construct into decoded image...");

        let (image_pixels, image_size, image_colour_type) = buffer_image.to_u8_pixels();

        Ok(
            DecodedImage::new(
                DecodedImageContent::Static(image_pixels),
                image_colour_type,
                image_format,
                image_size,
            )
        )
    }

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