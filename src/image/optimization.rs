use std::fmt::Display;

use image::DynamicImage;
use log::debug;
use display_info::DisplayInfo;

use crate::{error::{Error, Result}, notifier::NotifierAPI};

use super::{fast_downsample::fast_downsample, image::{Image, ImageSizeT}};

pub enum OptimizationProcessingMeat<'a> {
    ImageRS(&'a mut DynamicImage),
    Roseate(&'a mut Vec<u8>, &'a mut ImageSizeT, bool)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ImageOptimization {
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

impl ImageOptimization {
    pub fn id(&self) -> &str {
        match self {
            ImageOptimization::MonitorDownsampling(_) => "monitor-downsampling",
            ImageOptimization::DynamicDownsampling => "dynamic-downsampling",
            ImageOptimization::DynamicUpsampling => "dynamic-upsampling",
        }
    }
}

impl Display for ImageOptimization {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageOptimization::MonitorDownsampling(_) => write!(f, "Monitor Downsampling"),
            ImageOptimization::DynamicDownsampling => write!(f, "Dynamic Downsampling"),
            ImageOptimization::DynamicUpsampling => write!(f, "Dynamic Upsampling"),
        }
    }
}

impl Image {
    // TODO: Return actual error instead of "()".
    pub fn apply_optimizations(&self, notifier: &mut NotifierAPI, meat: OptimizationProcessingMeat) -> Result<()> {
        match meat {
            OptimizationProcessingMeat::ImageRS(dynamic_image) => {

                for optimization in self.optimizations.clone() {
                    notifier.set_loading(
                        Some(format!("Applying {:#} optimization...", optimization))
                    );
                    debug!("Applying '{:?}' optimization to image...", optimization);

                    *dynamic_image = match optimization {
                        ImageOptimization::MonitorDownsampling(marginal_allowance) => {
                            let (monitor_width, monitor_height) = get_monitor_size_before_egui_window()?;

                            let marginal_allowance_scale = marginal_allowance as f32 / 100.0;

                            let (width, height) = (
                                monitor_width as f32 * marginal_allowance_scale, monitor_height as f32 * marginal_allowance_scale
                            );

                            dynamic_image.resize(
                                width as u32,
                                height as u32,
                                image::imageops::FilterType::Lanczos3
                            )
                        },
                        _ => todo!()
                    };
                }

            },
            OptimizationProcessingMeat::Roseate(pixels, image_size, has_alpha) => {

                for optimization in self.optimizations.clone() {
                    (*pixels, *image_size) = match optimization {
                        ImageOptimization::MonitorDownsampling(marginal_allowance) => {
                            let (monitor_width, monitor_height) = get_monitor_size_before_egui_window()?;

                            let marginal_allowance_scale = marginal_allowance as f32 / 100.0;

                            let (width, height) = (
                                monitor_width as f32 * marginal_allowance_scale, monitor_height as f32 * marginal_allowance_scale
                            );

                            fast_downsample(
                                pixels.to_vec(), // this is pretty bad, .to_vec() will clone all these pixels. 
                                // We need to find a way to avoid this for memory spiking and performance sake.
                                &image_size,
                                (width as u32, height as u32), 
                                has_alpha
                            )
                        },
                        _ => todo!()
                    }
                }

            },
        }

        Ok(())
    }
}

// pub fn apply_image_optimizations(mut optimizations: Vec<ImageOptimization>, image_size: &ImageSize) -> Vec<ImageOptimization> {
//     let all_display_infos = DisplayInfo::all().expect(
//         "Failed to get information about your display monitor!"
//     );

//     // NOTE: I don't think the first monitor is always the primary and 
//     // if that is the case then we're gonna have a problem. (i.e images overly downsampled or not at all)
//     let primary_display_maybe = all_display_infos.first().expect(
//         "Uhhhhh, you don't have a monitor. WHAT!"
//     );

//     let marginal_allowance: f32 = 1.3;
//     // TODO: Make this adjustable in the config too as down sample strength.
//     // I'm still thinking about this so leave it out for now. ~ Goldy

//     let (width, height) = (
//         primary_display_maybe.width as f32 * marginal_allowance, 
//         primary_display_maybe.height as f32 * marginal_allowance
//     );

//     debug!(
//         "Display Size: {} x {}",
//         primary_display_maybe.width,
//         primary_display_maybe.height
//     );
//     debug!(
//         "Display Size + Downsample Marginal Allowance: {} x {}", width, height
//     );
//     debug!(
//         "Image Size: {} x {}",
//         image_size.width,
//         image_size.height
//     );

//     // If the image is a lot bigger than the user's monitor 
//     // then apply the downsample optimization for this image.
//     if image_size.width > width as usize && image_size.height > height as usize {
//         optimizations.push(ImageOptimization::Downsample(width as u32, height as u32));
//     }

//     optimizations
// }

// TODO: Return actual error instead of "()".
fn get_monitor_size_before_egui_window() -> Result<(u32, u32)> {
    let all_display_infos = DisplayInfo::all().expect(
        "Failed to get information about your display monitor!"
    );

    // NOTE: I don't think the first monitor is always the primary and 
    // if that is the case then we're gonna have a problem. (i.e images overly downsampled or not at all)
    match all_display_infos.first() {
        Some(primary_monitor_maybe) => {
            Ok((primary_monitor_maybe.width, primary_monitor_maybe.height))
        },
        None => Err(
            Error::MonitorNotFound(None)
        ),
    }
}