use std::{fs::{self, File}, io::{BufReader, Cursor}, path::{Path, PathBuf}, sync::Arc};

use log::debug;
use image::{ImageFormat, ImageReader};
use imagesize::ImageSize;

#[derive(Clone)]
pub struct Image {
    pub image_size: ImageSize,
    pub image_path: Arc<PathBuf>,
    pub image_bytes: Option<Arc<[u8]>>
}

#[derive(Debug)]
pub enum ImageOptimization {
    Downsample(u32, u32),
}

impl Image {
    pub fn from_path(path: &Path) -> Self {
        // I use imagesize crate to get the image size 
        // because it's A LOT faster as it only partially loads the image bytes.
        let image_size = imagesize::size(path).expect(
            "Failed to retrieve the dimensions of the image!"
        );

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