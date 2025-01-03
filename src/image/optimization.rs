use std::fmt::Display;

use image::DynamicImage;
use log::debug;
use imagesize::ImageSize;
use display_info::DisplayInfo;

use super::fast_downsample::fast_downsample;

#[derive(Debug)]
pub enum ImageOptimization {
    /// Downsamples the image to this width and height.
    /// 
    /// Images don't always have to be displayed at it's native resolution, 
    /// especially when the image is significantly bigger than our monitor 
    /// can even display so to save memory we downsample the image. Downsampling 
    /// decreases the memory usage of the image at the cost of time wasted actually 
    /// resizing the image. The bigger the image the more time it will take to downsample 
    /// but we think memory savings are more valuable. You can enable or disable downsampling
    /// in the config if you do not wish for such memory savings. Setting the overall optimization
    /// mode to speed ("s") will automatically disable this.
    /// 
    /// NOTE: "The image's aspect ratio is preserved. The image is scaled to the maximum 
    /// possible size that fits within the bounds specified by the width and height." ~ Image Crate
    Downsample(u32, u32),
}

impl ImageOptimization {
    pub fn apply_dynamic_image(&self, image: DynamicImage) -> DynamicImage {
        match self {
            ImageOptimization::Downsample(width, height) => {
                image.resize(
                    *width,
                    *height,
                    image::imageops::FilterType::Lanczos3
                )
            },
        }
    }

    pub fn apply_custom(&self, pixels: Vec<u8>, image_size: &ImageSize, has_alpha: bool) -> (Vec<u8>, (u32, u32)) {
        match self {
            ImageOptimization::Downsample(width, height) => {
                fast_downsample(pixels, image_size, (*width, *height), has_alpha)
            },
        }
    }
}

impl Display for ImageOptimization {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageOptimization::Downsample(_, _) => write!(f, "Downsample"),
        }
    }
}

pub fn apply_image_optimizations(mut optimizations: Vec<ImageOptimization>, image_size: &ImageSize) -> Vec<ImageOptimization> {
    let all_display_infos = DisplayInfo::all().expect(
        "Failed to get information about your display monitor!"
    );

    // NOTE: I don't think the first monitor is always the primary and 
    // if that is the case then we're gonna have a problem. (i.e images overly downsampled or not at all)
    let primary_display_maybe = all_display_infos.first().expect(
        "Uhhhhh, you don't have a monitor. WHAT!"
    );

    let marginal_allowance: f32 = 1.3;
    // TODO: Make this adjustable in the config too as down sample strength.
    // I'm still thinking about this. ~ Goldy

    let (width, height) = (
        primary_display_maybe.width as f32 * marginal_allowance, 
        primary_display_maybe.height as f32 * marginal_allowance
    );

    debug!(
        "Display Size: {} x {}",
        primary_display_maybe.width,
        primary_display_maybe.height
    );
    debug!(
        "Display Size + Downsample Marginal Allowance: {} x {}", width, height
    );
    debug!(
        "Image Size: {} x {}",
        image_size.width,
        image_size.height
    );

    // If the image is a lot bigger than the user's monitor 
    // then apply the downsample optimization for this image.
    if image_size.width > width as usize && image_size.height > height as usize {
        optimizations.push(ImageOptimization::Downsample(width as u32, height as u32));
    }

    optimizations
}