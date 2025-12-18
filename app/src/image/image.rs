use std::{collections::HashSet, fmt::Debug, fs::File, io::BufReader, path::PathBuf, sync::{Arc, Mutex}};

use log::debug;
use cirrus_egui::v1::notifier::Notifier;
use roseate_core::{backends::backend::DecodeBackend, decoded_image::{DecodedImage, ImageSize}, format::{ImageFormat, determine_image_format_and_size_from_header}, modifications::{ImageModification, ImageModifications}, reader::{ImageReader, ImageReaderData}};

use crate::{error::{Error, Result}, image::backend::DecodingBackend};

#[derive(Clone)]
pub struct Image {
    pub path: PathBuf,
    pub size: ImageSize,
    pub format: ImageFormat,
    pub decoded: Arc<Mutex<Option<DecodedImage>>>,

    last_modifications: ImageModifications,
}

impl Debug for Image {
    // I only want path, size, format and last modifications in the debug.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Image")
            .field("path", &self.path)
            .field("size", &self.size)
            .field("format", &self.format)
            .field("last_modifications", &self.last_modifications)
            .finish()
    }
}

impl Image {
    pub fn new(path: PathBuf) -> Result<Self> {
        if !path.exists() {
            return Err(
                Error::FileNotFound { path: path.to_string_lossy().to_string() }
            );
        }

        if path.extension().unwrap_or_default() == "svg" {
            // NOTE: this is experimental stuff, not final.
            // let size = determine_svg_size(&path);

            // return Ok(
            //     Self {
            //         path,
            //         size,
            //         format: ImageFormat::Svg,
            //         decoded: None
            //     }
            // );

            // We can't support SVG at all now as we no longer work with 
            // egui's image loader. https://github.com/cloudy-org/roseate/issues/89
            // 
            // I'm planning to create our own SVG renderer in the viewport by 
            // reading into egui's implementation of it in their loader.
            return Err(Error::SvgNotSupportedYet);
        }

        let (format, size) = determine_image_format_and_size_from_header(&path)?;

        Ok(
            Self {
                path,
                size,
                format,
                decoded: Arc::new(Mutex::new(None)),
                last_modifications: HashSet::default(),
            }
        )
    }

    pub fn load(&mut self, modifications: ImageModifications, backend: &DecodingBackend, reload: bool, notifier: &mut Notifier) -> Result<()> {
        notifier.set_loading(
            Some(
                format!(
                    "Preparing to {} image...",
                    match reload { true => "reload", false => "load" }
                )
            )
        );

        if reload && self.are_mods_the_same(&modifications) {
            debug!(
                "Image modifications were the same, rejecting reload..."
            );

            return Ok(());
        }

        let load_fresh_from_disk = match reload {
            true => self.are_mods_out_of_mem_bounds(&modifications),
            false => true,
        };

        // if we have already loaded this image and we can use the image in memory, the image reader will contain decoded image.
        let image_reader_data = self.get_image_reader_data(load_fresh_from_disk, notifier)?;
        let image_reader = ImageReader::new(image_reader_data, self.format.clone());

        notifier.set_loading(Some("Initializing decoder to use for decoding..."));
        let mut backend = backend.init_decoder(image_reader)?;

        notifier.set_loading(Some("Passing image modifications to decoder..."));
        self.last_modifications = modifications.clone();
        backend.modify(modifications);

        notifier.set_loading(Some("Decoding image..."));
        *self.decoded.lock().unwrap() = Some(backend.decode()?);

        debug!("Done decoding image!");

        notifier.unset_loading();

        Ok(())
    }

    /// Returns already decoded image from memory if it exists and if a fresh 
    /// image from disk is not required. Otherwise, in the case `fresh_from_disk` 
    /// is true or decoded image doesn't exist, a buf reader to the fresh image 
    /// on the disk is returned for us to decode later.
    fn get_image_reader_data(&mut self, fresh_from_disk: bool, notifier: &mut Notifier) -> Result<ImageReaderData> {
        if !fresh_from_disk {
            if let Some(decoded_image) = self.decoded.lock().unwrap().take() {
                return Ok(ImageReaderData::DecodedImage(decoded_image));
            }

            debug!("Decoded image is not currently loaded in memory, falling back to loading from disk...");
        }

        notifier.set_loading(Some("Opening image's file for reading..."));

        let file = File::open(&self.path)
            .map_err(
                |error| Error::ImageFileOpenFailure { error: error.to_string() }
            )?;

        notifier.unset_loading();

        debug!("Boxing image onto the heap to pass to buf reader...");

        Ok(ImageReaderData::BufReader(BufReader::new(Box::new(file))))
    }

    fn are_mods_out_of_mem_bounds(&mut self, modifications: &ImageModifications) -> bool {
        let require_resize = modifications.iter().find_map(|modification| {
            #[warn(irrefutable_let_patterns)]
            if let ImageModification::Resize(width, height) = modification {
                Some((*width, *height))
            } else {
                None
            }
        });

        let is_out_of_bounds = self.last_modifications.iter().any(|last_modification| {
            match last_modification {
                ImageModification::Resize(width, height) => {
                    match require_resize {
                        Some((new_width, new_height)) => {
                            // If this statement evaluates to true this 
                            // means we are being asked to resize the image upwards 
                            // to data we do not have in memory, hence we cannot use what's 
                            // in memory (these modifications are outside the memory bounds).
                            new_width > *width || new_height > *height
                        }
                        None => false,
                    }
                }
            }
        });

        debug!("Are any modifications out of memory bounds: {}", is_out_of_bounds);

        is_out_of_bounds
    }

    /// Check if modifications in both hash sets are the same.
    fn are_mods_the_same(&self, modifications: &ImageModifications) -> bool {
        if modifications.len() != self.last_modifications.len() {
            return false;
        }

        for modification in modifications {
            if !self.last_modifications.iter().any(|last_mod| modification == last_mod) {
                return false;
            }
        }

        true
    }
}