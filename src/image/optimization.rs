use std::{fmt::Display, io::Cursor};

use image::{codecs::{gif::GifEncoder, jpeg::JpegEncoder, png::PngEncoder, webp::WebPEncoder}, DynamicImage, ExtendedColorType, ImageDecoder, ImageEncoder};
use log::debug;

use crate::{error::{Error, Result}, image::image_formats::ImageFormat, notifier::NotifierAPI, utils::get_monitor_size_before_egui_window};

use super::{backends::ImageProcessingBackend, fast_downsample::fast_downsample, image::{Image, ImageSizeT}};

pub enum OptimizationProcessingMeat<'a> {
    ImageRS(&'a mut DynamicImage),
    Roseate(&'a mut Vec<u8>, &'a mut ImageSizeT, bool)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InitialImageOptimizations {
    /// Downsamples the image roughly to the resolution of your monitor.
    /// 
    /// Images don't always have to be displayed at their native resolution, 
    /// especially when the image is significantly bigger than your monitor 
    /// can even display, so to save memory we downsample the image. Downsampling 
    /// decreases the amount of memory used by rose of the image at the cost of time wasted actually 
    /// resizing the image. The bigger the image the more time it will take to downsample 
    /// but we think memory savings are more valuable. You can enable or disable downsampling
    /// in the config if you do not wish for such memory savings. Setting the overall optimization
    /// mode to speed ("s") will automatically disable this.
    /// 
    /// NOTE: "The image's aspect ratio is preserved. The image is scaled to the maximum 
    /// possible size that fits within the bounds specified by the width and height." ~ Image Crate
    MonitorDownsampling(u32),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EventImageOptimizations {
    /// Basically `MonitorDownsampling` but the image is 
    /// dynamically downsampled when full detail is no longer required 
    /// (for example, when the user zooms back out from a zoom that triggered `DynamicUpsampling`).
    DynamicDownsampling,
    /// `DynamicDownsampling` but the other way round. The image is upsampled relative to the 
    /// amount you zoomed into the image (zoom factor) all the way up to the image's native resolution.
    /// 
    /// The point of dynamic upsampling is to give back the detail that was lost from initially downsampling the image.
    DynamicUpsampling,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ImageOptimizations {
    Initial(InitialImageOptimizations),
    EventBased(EventImageOptimizations),
}

impl ImageOptimizations {
    pub fn id(&self) -> &str {
        match self {
            ImageOptimizations::Initial(optimization) => {
                match optimization {
                    InitialImageOptimizations::MonitorDownsampling(_) => "monitor-downsampling",
                }
            },
            ImageOptimizations::EventBased(optimization) => {
                match optimization {
                    EventImageOptimizations::DynamicDownsampling => "dynamic-downsampling",
                    EventImageOptimizations::DynamicUpsampling => "dynamic-upsampling",
                }
            }
        }
    }
}

impl Display for ImageOptimizations {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageOptimizations::Initial(optimization) => {
                match optimization {
                    InitialImageOptimizations::MonitorDownsampling(_) => write!(f, "Monitor Downsampling"),
                }
            },
            ImageOptimizations::EventBased(optimization) => {
                match optimization {
                    EventImageOptimizations::DynamicDownsampling => write!(f, "Dynamic Downsampling"),
                    EventImageOptimizations::DynamicUpsampling => write!(f, "Dynamic Upsampling"),
                }
            }
        }
    }
}

impl Image {
    pub(super) fn optimize_and_decode_image_to_buffer(
        &self,
        image_processing_backend: &ImageProcessingBackend,
        image_decoder: Box<dyn ImageDecoder>,
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
                self.apply_optimizations(
                    notifier,
                    OptimizationProcessingMeat::Roseate(
                        &mut pixels,
                        &mut actual_image_size,
                        has_alpha
                    )
                )?;

                notifier.set_loading(
                    Some("Encoding optimized image...".into())
                );
                debug!("Encoding optimized image from pixels to a buffer...");

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
                        self.apply_optimizations(
                            notifier,
                            OptimizationProcessingMeat::ImageRS(&mut dynamic_image)
                        )?;

                        notifier.set_loading(
                            Some("Encoding optimized image...".into())
                        );
                        debug!("Encoding optimized dynamic image into image buffer...");

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

    fn apply_optimizations(&self, notifier: &mut NotifierAPI, meat: OptimizationProcessingMeat) -> Result<()> {

        match meat {
            OptimizationProcessingMeat::ImageRS(dynamic_image) => {

                for optimization in self.optimizations.clone() {
                    self.debug_applying_optimization(notifier, &optimization);

                    if let ImageOptimizations::Initial(optimization) = optimization {
                        *dynamic_image = match optimization {
                            InitialImageOptimizations::MonitorDownsampling(marginal_allowance) => {
                                let (width, height) = get_monitor_downsampling_size(
                                    marginal_allowance, (dynamic_image.width(), dynamic_image.height())
                                );

                                dynamic_image.resize(
                                    width as u32,
                                    height as u32,
                                    image::imageops::FilterType::Lanczos3
                                )
                            },
                        }
                    };
                }
            },
            OptimizationProcessingMeat::Roseate(pixels, image_size, has_alpha) => {

                for optimization in self.optimizations.clone() {
                    self.debug_applying_optimization(notifier, &optimization);

                    if let ImageOptimizations::Initial(optimization) = optimization {
                        (*pixels, *image_size) = match optimization {
                            InitialImageOptimizations::MonitorDownsampling(marginal_allowance) => {
                                let (width, height) = get_monitor_downsampling_size(
                                    marginal_allowance, (image_size.0, image_size.1)
                                );

                                fast_downsample(
                                    pixels,
                                    &image_size,
                                    (width as u32, height as u32),
                                    has_alpha
                                )
                            }
                        }
                    }
                }

            },
        }

        Ok(())
    }

    /// Checks if the image has this TYPE of optimization applied, not the exact 
    /// optimization itself. Then it returns a reference to the exact optimization found.
    pub(super) fn has_optimization(&self, optimization: &ImageOptimizations) -> Option<&ImageOptimizations> {
        for applied_optimization in self.optimizations.iter() {
            if applied_optimization.id() == optimization.id() {
                return Some(applied_optimization);
            }
        }

        return None;
    }

    fn debug_applying_optimization(&self, notifier: &mut NotifierAPI, optimization: &ImageOptimizations) {
        notifier.set_loading(
            Some(format!("Applying {:#} optimization...", optimization))
        );
        debug!("Applying '{:?}' optimization to image...", optimization);
    }
}

// TODO: when I have a centralized place for individual optimization logic move this there.
pub fn get_monitor_downsampling_size(marginal_allowance: u32, image_size: (u32, u32)) -> (u32, u32) {
    // marginal_allowance is supposed to be a f32 but instead 
    // it's a u32 hence all it's units have been shifted forward one.
    // 
    // E.g. "130" is "1.3"
    let marginal_allowance_scale = marginal_allowance as f32 / 100.0;

    debug!(
        "Image Size: {} x {}", image_size.0, image_size.1
    );

    let (monitor_width, monitor_height) = get_monitor_size_before_egui_window()
        .unwrap_or((1920, 1080));

    debug!(
        "Display (Monitor) Size: {} x {}", monitor_width, monitor_height
    );

    let (width, height) = (
        (monitor_width as f32 * marginal_allowance_scale) as u32,
        (monitor_height as f32 * marginal_allowance_scale) as u32
    );

    debug!(
        "Display + Monitor Downsample Marginal Allowance ({}): {} x {}",
        marginal_allowance_scale, width, height
    );

    (width, height)
}