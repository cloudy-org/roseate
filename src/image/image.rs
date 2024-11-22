use std::{fs::{self, File}, io::{BufReader, Cursor, Read}, path::{Path, PathBuf}, sync::{Arc, Mutex}};

use log::debug;
use eframe::egui::Vec2;
use imagesize::ImageSize;
use svg_metadata::Metadata;
use display_info::DisplayInfo;
use image::{codecs::{gif::{GifDecoder, GifEncoder}, jpeg::{JpegDecoder, JpegEncoder}, png::{PngDecoder, PngEncoder}, webp::{WebPDecoder, WebPEncoder}}, DynamicImage, ExtendedColorType, ImageDecoder, ImageEncoder, ImageResult};

use crate::{error::Error, notifier::NotifierAPI};

use super::{backends::ImageProcessingBackend, image_formats::ImageFormat, optimization::ImageOptimization};

#[derive(Clone)]
pub struct Image {
    pub image_size: ImageSize,
    pub image_format: ImageFormat,
    pub image_path: Arc<PathBuf>,
    pub image_bytes: Arc<Mutex<Option<Arc<[u8]>>>>,
    // Look! I know you see that type above me but just  
    // so you know, I'm NOT crazy... well not yet at least...
    // 
    // Anyways, as you can see, `image_bytes` is an `Arc<Mutex<Option<Arc<[u8]>>>>` 
    // this is because we need to be able to manipulate this under a thread so we can load
    // images in a background thread (see https://github.com/cloudy-org/roseate/issues/24).
    // 
    // The first Arc allows us to share the SAME image_bytes safely across threads even when we 
    // image.clone() that bitch, while Mutex ensures that only one thread accesses or modifies the image 
    // bytes and also SO THE RUST COMPILER CAN SHUT THE FUCK UP.. YES I KNOW THAT IT'S UNSAFE BECAUSE ANOTHER 
    // THREAD CAN FUCK IT UP BUT YOU DO REALISE MY PROGRAM IS SMART ENOUGH TO NOT DO THAT... uhmmm uhmmm... anyways... 
    // I use an Option because an image that is not yet loaded will have no bytes in memory and the second Arc is there
    // so we can image.clone() and not be doubling the image bytes in memory and turn into the next Google Chrome web browser. 💀
    // 
    // Kind regards,
    // Goldy
}

impl Image {
    // TODO: Return result instead of panicking (e.g. right now if you 
    // open an unsupported file type roseate will crash because we panic at line 60).
    pub fn from_path(path: &Path) -> Result<Self, Error> {
        // Changed this to unwrap_or_default so it returns an empty 
        // string ("") and doesn't panic if a file has no extension. I need to begin adding tests.
        let extension = path.extension().unwrap_or_default();

        let (image_size, image_format) = if extension == "svg" {
            (
                get_svg_image_size(&path),
                ImageFormat::Svg
            )
        } else {
            // I use 'imagesize' crate to get the image size and correct image
            // format because it's A LOT faster as it only partially loads the image bytes.

            let mut buffer = [0u8; 16];
            let number_of_bytes_read = File::open(&path)
                .expect("Failed to open file to get image type!") // we can expect here as we currently already check if the file exists prior.
                .read(&mut buffer)
                .unwrap(); // should we unwrap, I think this has a likelihood of failing.

            let image_format = match imagesize::image_type(&buffer[..number_of_bytes_read]) {
                Ok(image_size_image_type) => {
                    match ImageFormat::try_from(image_size_image_type) {
                        Ok(value) => value,
                        Err(error) => {
                            return Err(
                                Error::FailedToInitImage(
                                    Some(error.message()), path.to_path_buf(), error.message()
                                )
                            )
                        },
                    }
                },
                Err(error) => {
                    return Err(
                        Error::FailedToInitImage(
                            Some(error.to_string()),
                            path.to_path_buf(),
                            "Failed to retrieve image type!".to_string()
                        )
                    )
                },
            };

            let image_size = match imagesize::size(&path) {
                Ok(value) => value,
                Err(error) => {
                    return Err(
                        Error::FailedToInitImage(
                            Some(error.to_string()),
                            path.to_path_buf(),
                            "Failed to retrieve image dimensions!".to_string()
                        )
                    );
                },
            };

            (image_size, image_format)
        };

        Ok(
            Self {
                image_size,
                image_format,
                image_path: Arc::new(path.to_owned()),
                image_bytes: Arc::new(Mutex::new(None))
            }
        )
    }

    pub fn load_image(&mut self, optimizations: &[ImageOptimization], notifier: &mut NotifierAPI, image_processing_backend: &ImageProcessingBackend) -> Result<(), Error> {
        if optimizations.is_empty() {
            debug!("No optimizations were set so loading with fs::read instead...");

            let mut image_bytes_lock = self.image_bytes.lock().unwrap();

            // TODO: return Error instead of panic.
            *image_bytes_lock = Some(
                Arc::from(fs::read(self.image_path.as_ref()).expect("Failed to read image with fs::read!"))
            );

            return Ok(()); // I avoid image crate here as loading the bytes with fs::read is 
            // A LOT faster and no optimizations need to be done so we don't need image crate.
        }

        let (mut actual_width, mut actual_height) = (
            self.image_size.width as u32,
            self.image_size.height as u32
        );

        notifier.set_loading(Some("Opening file...".into()));
        debug!("Opening file into buf reader for image crate to read...");

        let image_file = match File::open(self.image_path.as_ref()) {
            Ok(file) => file,
            Err(error) => {
                return Err(
                    Error::FileNotFound(
                        Some(error.to_string()),
                        self.image_path.to_path_buf(),
                        "The file we're trying to load does not exist any more!".to_string()
                    )
                )
            },
        };

        let image_buf_reader = BufReader::new(image_file); // apparently this is faster for larger files as 
        // it avoids loading files line by line hence less system calls to the disk. (EDIT: I'm defiantly noticing a speed difference)

        notifier.set_loading(Some("Passing image to image decoder...".into()));
        debug!("Loading image buf reader into image decoder so optimizations can be applied to pixels...");

        let image_decoder: Box<dyn ImageDecoder> = match self.image_format {
            ImageFormat::Png => Box::new(PngDecoder::new(image_buf_reader).unwrap()),
            ImageFormat::Jpeg => Box::new(JpegDecoder::new(image_buf_reader).unwrap()),
            ImageFormat::Svg => panic!("SVGs cannot be loaded with optimizations at the moment!"),
            ImageFormat::Gif => Box::new(GifDecoder::new(image_buf_reader).unwrap()),
            ImageFormat::Webp => Box::new(WebPDecoder::new(image_buf_reader).unwrap()),
        };

        let image_colour_type = image_decoder.color_type();

        let mut optimized_image_buffer: Vec<u8> = Vec::new();

        notifier.set_loading(Some("Decoding image...".into()));

        let image_result: ImageResult<()> = match image_processing_backend {
            ImageProcessingBackend::Roseate => {
                let mut pixels = vec![0; image_decoder.total_bytes() as usize];

                debug!("Decoding pixels from image using image decoder...");
                image_decoder.read_image(&mut pixels).unwrap();

                for optimization in optimizations {
                    notifier.set_loading(
                        Some(format!("Applying {:#} optimization...", optimization))
                    );
                    debug!("Applying '{:?}' optimization to image...", optimization);

                    (pixels, (actual_width, actual_height)) = optimization.apply_custom(pixels, &self.image_size);
                }

                notifier.set_loading(
                    Some("Encoding optimized image...".into())
                );
                debug!("Encoding optimized image from pixels to a buffer...");

                match self.image_format {
                    ImageFormat::Png => {
                        PngEncoder::new(&mut optimized_image_buffer).write_image(
                            &pixels,
                            actual_width,
                            actual_height,
                            ExtendedColorType::Rgb8
                        )
                    },
                    ImageFormat::Jpeg => {
                        JpegEncoder::new(&mut optimized_image_buffer).write_image(
                            &pixels,
                            actual_width,
                            actual_height,
                            image_colour_type.into()
                        )
                    },
                    ImageFormat::Svg => {
                        PngEncoder::new(&mut optimized_image_buffer).write_image(
                            &pixels,
                            actual_width,
                            actual_height,
                            image_colour_type.into()
                        )
                    },
                    ImageFormat::Gif => {
                        GifEncoder::new(&mut optimized_image_buffer).encode(
                            &pixels,
                            actual_width,
                            actual_height,
                            image_colour_type.into()
                        )
                    },
                    ImageFormat::Webp => {
                        WebPEncoder::new_lossless(&mut optimized_image_buffer).write_image(
                            &pixels,
                            actual_width,
                            actual_height,
                            image_colour_type.into()
                        )
                    },
                }
            },
            ImageProcessingBackend::ImageRS => {
                debug!("Decoding image into dynamic image...");

                let result = DynamicImage::from_decoder(image_decoder);

                match result {
                    Ok(mut dynamic_image) => {
                        for optimization in optimizations {
                            notifier.set_loading(
                                Some(format!("Applying {:#} optimization...", optimization))
                            );
                            debug!("Applying '{:?}' optimization to image...", optimization);

                            dynamic_image = optimization.apply_dynamic_image(dynamic_image);
                        }

                        notifier.set_loading(
                            Some("Encoding optimized image...".into())
                        );
                        debug!("Encoding optimized dynamic image into image buffer...");

                        dynamic_image.write_to(
                            &mut Cursor::new(&mut optimized_image_buffer),
                            self.image_format.to_image_rs_format()
                        )
                    },
                    Err(error) => Err(error)
                }
            }
        };

        if let Err(image_error) = image_result {
            let error = Error::FailedToApplyOptimizations(
                Some(image_error.to_string()),
                "Failed to decode and load image to apply optimizations!".to_string()
            );

            // warn the user that optimizations failed to apply.
            notifier.toasts.lock().unwrap()
                .toast_and_log(error.into(), egui_notify::ToastLevel::Error);

            // load image without optimizations
            let result = self.load_image(&[], notifier, image_processing_backend);

            match result {
                Ok(_) => return Ok(()),
                Err(error) => return Err(error),
            }
        }

        *self.image_bytes.lock().unwrap() = Some(Arc::from(optimized_image_buffer));

        Ok(())
    }
}

fn get_primary_display_info() -> DisplayInfo {
    let all_display_infos = DisplayInfo::all().expect(
        "Failed to get information about your display monitor!"
    );

    // NOTE: I don't think the first monitor is always the primary and 
    // if that is the case then we're gonna have a problem. (i.e images overly downsampled or not at all)
    let primary_display_maybe = all_display_infos.first().expect(
        "Uhhhhh, you don't have a monitor. WHAT!"
    );

    primary_display_maybe.clone()
}

fn get_svg_image_size(path: &Path) -> ImageSize {
    let metadata = Metadata::parse_file(path).expect(
        "Failed to parse metadata of the svg file!"
    );

    let width = metadata.width().expect("Failed to get SVG width");
    let height = metadata.height().expect("Failed to get SVG height");

    let display_info = get_primary_display_info();

    let image_to_display_ratio = Vec2::new(width as f32, height as f32) /
        Vec2::new(display_info.width as f32, display_info.height as f32);

    // Temporary solution to give svg images a little bit higher quality.
    ImageSize {
        width: (width * (1.0 + (1.0 - image_to_display_ratio.x)) as f64) as usize,
        height: (height * (1.0 + (1.0 - image_to_display_ratio.y)) as f64) as usize 
    }
}