use std::{fs::{self, File}, io::{BufReader, Cursor}, path::{Path, PathBuf}, sync::Arc};

use log::debug;
use imagesize::ImageSize;
use svg_metadata::Metadata;
use display_info::DisplayInfo;
use image::{ImageFormat, ImageReader};

#[derive(Clone)]
pub struct Image {
    pub image_size: ImageSize,
    pub image_path: Arc<PathBuf>,
    pub image_bytes: Option<Arc<[u8]>>
}

#[derive(Debug)]
pub enum ImageOptimization {
    /// Downsamples the image to this width and height.
    /// 
    /// Images don't always have to be displayed at it's native resolution, 
    /// especially when the image is significantly bigger than our monitor 
    /// can even display so to save memory we downsample the image. Downsampling 
    /// decreases the memory usage of the image at the cost of time wasted actually 
    /// resizing the image. The bigger the image the more time it will take to downsample 
    /// but the memory savings are very valuable.
    /// 
    /// NOTE: "The image's aspect ratio is preserved. The image is scaled to the maximum 
    /// possible size that fits within the bounds specified by the width and height." ~ Image Crate
    Downsample(u32, u32),
}

impl Image {
    pub fn from_path(path: &Path) -> Self {
        // Changed this to unwrap_or_default so it returns an empty 
        // string ("") and doesn't panic if a file has no extension. I need to begin adding tests.
        let extension = path.extension().unwrap_or_default();

        let image_size: ImageSize = if extension == "svg" {
            get_svg_image_size(&path)
        } else {
            // I use 'imagesize' crate to get the image size 
            // because it's A LOT faster as it only partially loads the image bytes.
            imagesize::size(path).expect(
                "Failed to retrieve the dimensions of the image!"
            )
        };

        Self {
            image_size,
            image_path: Arc::new(path.to_owned()),
            image_bytes: None
        }
    }

    pub fn load_image(&mut self, optimizations: &[ImageOptimization]) {
        if optimizations.is_empty() {
            debug!("No optimizations were set so loading with fs::read instead...");

            self.image_bytes = Some(
                Arc::from(fs::read(self.image_path.as_ref()).expect("Failed to read image with fs::read!"))
            );
            return; // I avoid image crate here as loading the bytes with fs::read is 
            // A LOT faster and no optimizations need to be done so we don't need image crate.
        }

        debug!("Opening file into buf reader...");

        let image_file = File::open(self.image_path.as_ref()).expect(
            &format!("Failed to open file for the image '{}'", self.image_path.to_string_lossy())
        );
        let image_buf_reader = BufReader::new(image_file); // apparently this is faster for larger files as 
        // it avoids loading files line by line hence less system calls to the disk. (EDIT: I'm defiantly notice a speed difference)

        debug!("Loading image into image crate DynamicImage so optimizations can be applied...");

        let mut image = ImageReader::new(image_buf_reader)
            .with_guessed_format().unwrap().decode().expect(
            "Failed to decode and load image with image crate to apply optimizations!"
        );

        for optimization in optimizations {
            debug!("Applying '{:?}' optimization to image...", optimization);

            match optimization {
                ImageOptimization::Downsample(width, height) => {
                    image = image.resize(
                        *width,
                        *height,
                        image::imageops::FilterType::Lanczos3
                    );
                },
            }
        }

        // TODO: I think writing the modified image into this buffer will make the memory usage 
        // spike quite a lot as it will basically be duplicating it as we already the unmodified image 
        // in self.image_bytes. Maybe we should clear self.image_bytes before we write the modified image to the buffer.
        let mut buffer: Vec<u8> = Vec::new();

        image.write_to(&mut Cursor::new(&mut buffer), ImageFormat::WebP).expect(
            "Failed to write optimized image to buffer!"
        );

        self.image_bytes = Some(Arc::from(buffer));
    }
}

// NOTE: should this be here? Don't know.
pub fn apply_image_optimizations(mut optimizations: Vec<ImageOptimization>, image_size: &ImageSize) -> Vec<ImageOptimization> {
    let all_display_infos = DisplayInfo::all().expect(
        "Failed to get information about your display monitor!"
    );

    // NOTE: I don't think the first monitor is always the primary and 
    // if that is the case then we're gonna have a problem. (i.e images overly downsampled or not at all)
    let primary_display_maybe = all_display_infos.first().expect(
        "Uhhhhh, you don't have a monitor. WHAT!"
    );

    let marginal_allowance: f32 = 1.3; // TODO: Make this adjustable in the config too as down sample strength.

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

fn get_svg_image_size(path: &Path) -> ImageSize {
    let metadata = Metadata::parse_file(path).expect(
        "Failed to parse metadata of the svg file!"
    );

    let width = metadata.width().expect("Failed to get SVG width");
    let height = metadata.height().expect("Failed to get SVG height");

    ImageSize {
        width: width as usize,
        height: height as usize
    }
}