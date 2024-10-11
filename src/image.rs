use std::{fs, io::Cursor, path::{Path, PathBuf}, sync::Arc};

use log::debug;
use image::{ImageFormat, ImageReader};
use imagesize::ImageSize;

#[derive(Clone)]
pub struct Image {
    pub image_size: ImageSize,
    pub image_path: PathBuf,
    pub image_bytes: Option<Arc<[u8]>>
}

#[derive(Debug)]
pub enum ImageOptimization {
    Downsample(u32, u32),
}

impl Image {
    pub fn from_path(path: &Path) -> Self {
        let image_size = imagesize::size(path).expect(
            "Failed to retrieve the dimensions of the image!"
        );

        Self {
            image_size,
            image_path: path.to_owned(),
            image_bytes: None
        }
    }

    pub fn load_image(&mut self, optimizations: &[ImageOptimization]) {
        if optimizations.is_empty() {
            debug!("No optimizations were set so loading with fs:read instead...");

            self.image_bytes = Some(
                Arc::from(fs::read(self.image_path.clone()).expect("Failed to read image with fs::read!"))
            );
            return;
        }

        debug!("Loading image into image crate DynamicImage so optimizations can be applied...");

        let mut image = ImageReader::open(self.image_path.clone()).expect(
            "Failed to open image with image crate to apply optimizations!"
        ).decode().expect("Failed to decode and load image with image crate to apply optimizations!");

        for optimization in optimizations {
            debug!("Applying '{:?}' optimization to image...", optimization);

            match optimization {
                ImageOptimization::Downsample(width, height) => {
                    image = image.resize(
                        *width,
                        *height,
                        image::imageops::FilterType::Gaussian
                    );
                },
            }
        }

        let mut buffer: Vec<u8> = Vec::new();

        image.write_to(&mut Cursor::new(&mut buffer), ImageFormat::WebP).expect(
            "Failed to write optimized image to buffer!"
        );

        self.image_bytes = Some(Arc::from(buffer));
    }

}