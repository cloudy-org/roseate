use std::{collections::HashSet, io::BufReader};

use image::{
    AnimationDecoder, ImageDecoder, ImageError, codecs::{
        gif::GifDecoder, jpeg::JpegDecoder, png::PngDecoder, tiff::TiffDecoder, webp::WebPDecoder,
    },
};
use log::debug;

use crate::{
    backends::{backend::DecodeBackend, image_rs::buffer_image::{BufferImage, BufferImageVariant}}, colour_type::ImageColourType, decoded_image::{DecodedImage, DecodedImageContent, ImageSize, Pixels}, error::{Error, Result}, format::ImageFormat, image_info::metadata::ImageMetadata, modifications::{ImageModification, ImageModifications}, reader::{ImageReader, ImageReaderData, ReadSeek}
};

mod colour;
mod buffer_image;
mod modifications;

// TODO: Fill with debug logs

enum Decoder {
    Png(PngDecoder<BufReader<Box<dyn ReadSeek>>>),
    Jpeg(JpegDecoder<BufReader<Box<dyn ReadSeek>>>),
    Webp(WebPDecoder<BufReader<Box<dyn ReadSeek>>>),
    Gif(GifDecoder<BufReader<Box<dyn ReadSeek>>>),
    Tiff(TiffDecoder<BufReader<Box<dyn ReadSeek>>>),
}

enum Buffer {
    Image(BufferImage),
    Animation((Vec<(BufferImage, f32)>, ImageSize, ImageColourType)),
}

enum Source {
    Decoder(Decoder),
    Buffer(Buffer),
}

pub struct ImageRSBackend {
    source: Source,
    modifications: ImageModifications,
    image_exif_chunk: Option<Vec<u8>>,
    image_format: ImageFormat,
}

impl DecodeBackend for ImageRSBackend {
    fn from_reader(image_reader: ImageReader) -> Result<Self> {
        match image_reader.data {
            ImageReaderData::BufReader(buf_reader) => {
                log::debug!("Initializing image-rs backend decoders with buf reader...");

                let error_func = |error: ImageError| Error::DecoderInitFailure {
                    error: error.to_string(),
                };

                let mut image_decoder = match image_reader.image_format {
                    ImageFormat::Gif => Decoder::Gif(GifDecoder::new(buf_reader).map_err(error_func)?),
                    ImageFormat::Png => Decoder::Png(PngDecoder::new(buf_reader).map_err(error_func)?),
                    ImageFormat::Jpeg => Decoder::Jpeg(JpegDecoder::new(buf_reader).map_err(error_func)?),
                    ImageFormat::Webp => Decoder::Webp(WebPDecoder::new(buf_reader).map_err(error_func)?),
                    ImageFormat::Tiff => Decoder::Tiff(TiffDecoder::new(buf_reader).map_err(error_func)?),
                    unsupported_format => {
                        return Err(Error::DecoderNotSupported {
                            image_format: unsupported_format.to_string(),
                            backend: String::from("image-rs"),
                        });
                    }
                };

                let exif_chunk = match &mut image_decoder {
                    Decoder::Png(png_decoder) => png_decoder.exif_metadata(),
                    Decoder::Jpeg(jpeg_decoder) => jpeg_decoder.exif_metadata(),
                    Decoder::Webp(web_pdecoder) => web_pdecoder.exif_metadata(),
                    Decoder::Gif(gif_decoder) => gif_decoder.exif_metadata(),
                    Decoder::Tiff(tiff_decoder) => tiff_decoder.exif_metadata(),
                }.map_err(|error| Error::DecoderRetrieveExifFailure { error: error.to_string() })?;

                Ok(
                    Self {
                        source: Source::Decoder(image_decoder),
                        modifications: HashSet::new(),
                        image_exif_chunk: exif_chunk,
                        image_format: image_reader.image_format
                    }
                )
            },
            ImageReaderData::DecodedImage(decoded_image) => {
                log::debug!("Initializing image-rs backend from already decoded image...");

                match decoded_image.content {
                    DecodedImageContent::Static(pixels) => {
                        // If we're coming from a decoded image it won't support any other pixel variants other than u8.
                        let image_buffer = BufferImage::from_u8_pixels(
                            pixels,
                            decoded_image.size,
                            decoded_image.colour_type,
                        )?;

                        Ok(
                            Self {
                                source: Source::Buffer(Buffer::Image(image_buffer)),
                                modifications: HashSet::new(),
                                image_exif_chunk: None, // decoded image should 
                                // contain it so we don't need the chunk no more
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
                                decoded_image.colour_type,
                            )?;

                            animated_buffers.push((image_buffer, delay));
                        }

                        Ok(
                            Self {
                                source: Source::Buffer(
                                    Buffer::Animation((
                                        animated_buffers,
                                        decoded_image.size,
                                        decoded_image.colour_type
                                    ))
                                ),
                                modifications: HashSet::new(),
                                image_exif_chunk: None,
                                image_format: image_reader.image_format
                            }
                        )
                    }
                }
            }
        }
    }

    fn modify<I>(&mut self, modifications: I)
    where
        I: IntoIterator<Item = ImageModification>,
    {
        self.modifications.extend(modifications);
    }

    fn decode(self) -> Result<DecodedImage> {
        match self.source {
            Source::Decoder(decoder) => match decoder {
                Decoder::Png(png_decoder) => {
                    let has_animation = png_decoder.is_apng().map_err(|error| {
                        Error::DecoderAnimationCheckFailure {
                            error: error.to_string(),
                        }
                    })?;

                    match has_animation {
                        true => {
                            let apng_decoder = png_decoder.apng()
                                .expect("We should have been given the image-rs APNG Decoder but we weren't!");

                            Self::decode_animated_image(
                                apng_decoder,
                                self.modifications,
                                self.image_format,
                                self.image_exif_chunk
                            )
                        },
                        false => Self::decode_image(
                            png_decoder,
                            self.modifications,
                            self.image_format,
                            self.image_exif_chunk
                        )
                    }
                },
                Decoder::Webp(webp_decoder) => {
                    match webp_decoder.has_animation() {
                        true => Self::decode_animated_image(
                            webp_decoder,
                            self.modifications,
                            self.image_format,
                            self.image_exif_chunk
                        ),
                        false => Self::decode_image(
                            webp_decoder,
                            self.modifications,
                            self.image_format,
                            self.image_exif_chunk
                        ),
                    }
                },
                Decoder::Gif(gif_decoder) => Self::decode_animated_image(
                    gif_decoder,
                    self.modifications,
                    self.image_format,
                    self.image_exif_chunk
                ),
                Decoder::Jpeg(jpeg_decoder) => Self::decode_image(
                    jpeg_decoder,
                    self.modifications,
                    self.image_format,
                    self.image_exif_chunk
                ),
                Decoder::Tiff(tiff_decoder) => {
                    Self::decode_image(
                        tiff_decoder,
                        self.modifications,
                        self.image_format,
                        self.image_exif_chunk
                    )
                }
            },
            Source::Buffer(buffer) => {
                log::debug!(
                    "Image already decoded and constructed as image-rs image buffer, applying modifications..."
                );

                match buffer {
                    Buffer::Image(mut buffer_image) => {
                        Self::apply_modifications_to_buffer_image(self.modifications, &mut buffer_image);

                        let (pixels, size, colour_type) = buffer_image.to_u8_pixels();

                        Ok(
                            DecodedImage::new(
                                size,
                                self.image_format,
                                colour_type,
                                Self::get_decoded_image_metadata(self.image_exif_chunk),
                                DecodedImageContent::Static(pixels),
                            )
                        )
                    },
                    Buffer::Animation((buffer_image_frames, size, colour_type)) => {
                        let mut animated_pixels = Vec::new();

                        for (index, (mut buffer_image, delay)) in buffer_image_frames.into_iter().enumerate() {
                            debug!("Applying modifications to frame {}...", index);

                            Self::apply_modifications_to_buffer_image(
                                self.modifications.clone(),
                                &mut buffer_image,
                            );

                            let (pixels, _, _) = buffer_image.to_u8_pixels();

                            animated_pixels.push((pixels, delay));
                        }

                        Ok(
                            DecodedImage::new(
                                size,
                                self.image_format,
                                colour_type,
                                Self::get_decoded_image_metadata(self.image_exif_chunk),
                                DecodedImageContent::Animated(animated_pixels),
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
        image_format: ImageFormat,
        image_exif_chunk: Option<Vec<u8>>,
    ) -> Result<DecodedImage> {
        let mut image_size_and_metadata: Option<(ImageSize, ImageMetadata)> = None;

        let init_size_and_metadata = |size: ImageSize| -> (ImageSize, ImageMetadata) {
            (size, Self::get_decoded_image_metadata(image_exif_chunk.to_owned()))
        };

        let mut image_pixels: Vec<(Pixels, f32)> = Vec::new();

        let perform_modifications = !modifications.is_empty();

        for frame_result in animation_decoder.into_frames() {
            let frame = frame_result.map_err(
                // NOTE: I might change this to a less generic error.
                |error| Error::DecodingFailure {
                    error: format!(
                        "Image-rs decoder failed to decode animated frame: {}",
                        error.to_string()
                    ),
                },
            )?;

            let (numerator, denominator) = frame.delay().numer_denom_ms();
            let delay_seconds = (numerator as f32 / denominator as f32) / 1000.0;

            let image_buffer = frame.into_buffer();

            let size = image_buffer.dimensions();

            let mut buffer_image = BufferImage {
                size,
                colour_type: ImageColourType::Rgba8,
                variant: BufferImageVariant::Rgba8(image_buffer),
            };

            if perform_modifications {
                Self::apply_modifications_to_buffer_image(modifications.clone(), &mut buffer_image);
            }

            let (pixels, size, _) = buffer_image.to_u8_pixels();

            image_size_and_metadata.get_or_insert_with(
                || init_size_and_metadata(size)
            );

            image_pixels.push((pixels, delay_seconds));
        }

        let (info, metadata) = match image_size_and_metadata {
            Some(value) => value,
            None => return Err(Error::AnimatedImageHasNoFrames),
        };

        Ok(
            DecodedImage::new(
                info,
                image_format,
                ImageColourType::Rgba8,
                metadata,
                DecodedImageContent::Animated(image_pixels),
            )
        )
    }

    fn decode_image<T: ImageDecoder>(
        image_decoder: T,
        modifications: ImageModifications,
        image_format: ImageFormat,
        image_exif_chunk: Option<Vec<u8>>,
    ) -> Result<DecodedImage> {
        log::debug!("Decoding image with image-rs decoder...");

        let mut image_pixels: Vec<u8> = vec![0; image_decoder.total_bytes() as usize];

        let image_size = image_decoder.dimensions();
        let image_colour_type = ImageColourType::try_from(image_decoder.color_type())?;

        if let Err(error) = image_decoder.read_image(&mut image_pixels) {
            // TODO: map extract image rs error to a roseate-core error.
            return Err(
                // NOTE: I might change this to a less generic error.
                Error::DecodingFailure {
                    error: format!(
                        "Image-rs decoder failed to decode image to pixels: {}",
                        error.to_string()
                    ),
                },
            );
        }

        log::debug!("Image-rs decoder successfully decoded to pixels...");

        let metadata = Self::get_decoded_image_metadata(image_exif_chunk);

        if modifications.is_empty() {
            log::debug!(
                "No image modifications so we're constructing decoded image directly from image pixels..."
            );

            return Ok(
                DecodedImage::new(
                    image_size,
                    image_format,
                    image_colour_type,
                    metadata,
                    DecodedImageContent::Static(image_pixels),
                )
            );
        }

        log::debug!(
            "We have image modifications. Constructing image-rs image buffer to apply modifications..."
        );

        let mut buffer_image = BufferImage::from_u8_pixels(
            image_pixels,
            image_size,
            image_colour_type,
        )?;

        log::debug!("Image buffer constructed, applying modifications...");

        Self::apply_modifications_to_buffer_image(modifications, &mut buffer_image);

        log::debug!("Converting image buffer back to pixels to construct into decoded image...");

        let (image_pixels, image_size, image_colour_type) = buffer_image.to_u8_pixels();

        Ok(
            DecodedImage::new(
                image_size,
                image_format,
                image_colour_type,
                metadata,
                DecodedImageContent::Static(image_pixels),
            )
        )
    }

    fn get_decoded_image_metadata(image_exif_chunk: Option<Vec<u8>>) -> ImageMetadata {
        match image_exif_chunk {
            Some(exif_chunk) => match ImageMetadata::new(exif_chunk) {
                Ok(metadata) => metadata,
                Err(error) => {
                    log::warn!("{}", error);

                    ImageMetadata::default()
                },
            },
            None => ImageMetadata::default(),
        }
    }
}
