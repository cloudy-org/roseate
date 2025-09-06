use std::{collections::HashSet, fs::File, hash::{DefaultHasher, Hasher}, io::{BufReader, Read}, path::{Path, PathBuf}, sync::{Arc, Mutex}};

use std::hash::Hash;
use log::debug;
use svg_metadata::Metadata;

use crate::{error::{Error, Result}, image::decode::DecodedImage, monitor_size::MonitorSize, notifier::NotifierAPI};

use super::{backends::ImageProcessingBackend, image_data::{ImageColourType, ImageData}, image_formats::ImageFormat, modifications::ImageModifications};

pub type ImageSizeT = (u32, u32);

// trait ReadAndSeek: Read + Seek {}
// not to be confused with hide and seek.
// impl<T: Read + Seek> ReadAndSeek for T {}

#[derive(Clone)]
pub struct Image {
    pub image_size: ImageSizeT,
    pub image_format: ImageFormat,
    pub image_path: Arc<PathBuf>,
    pub image_data: Arc<Mutex<Option<ImageData>>>,
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
    current_modifications: Arc<Mutex<HashSet<ImageModifications>>>
}

impl Hash for Image {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (*self.image_path).hash(state);

        if let Some(image_data) = self.image_data.lock().unwrap().as_ref() {
            match image_data {
                ImageData::Pixels(pixels_data) => pixels_data.0.len().hash(state),
                ImageData::StaticBytes(image_bytes) => image_bytes.len().hash(state),
            }
        }
    }
}

impl Image {
    pub fn from_path(path: &Path) -> Result<Self> {
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

            ((image_size.width as u32, image_size.height as u32), image_format)
        };

        Ok(
            Self {
                image_size,
                image_format,
                image_path: Arc::new(path.to_owned()),
                image_data: Arc::new(Mutex::new(None)),
                current_modifications: Arc::new(Mutex::new(HashSet::new()))
            }
        )
    }

    /// Reloads image pretty fast by using image_bytes in memory when possible.
    /// Falls back to disk if the modifications make it impossible to load from memory.
    pub fn reload_image(
        &mut self,
        notifier: &mut NotifierAPI,
        modifications: HashSet<ImageModifications>,
        image_processing_backend: &ImageProcessingBackend
    ) -> Result<()> {
        if self.are_modifications_the_same(&modifications, &self.current_modifications.lock().unwrap()) {
            debug!(
                "Modifications were the same so there's no \
                reason to reload this image hence we are skipping..."
            );

            return Ok(());
        }

        let load_from_disk = self.are_modifications_outside_memory_bounds(&modifications);

        notifier.set_loading_and_log(
            Some("Preparing image to be reloaded...".into())
        );

        let current_modifications = modifications.clone();

        let arc_pixels: (Arc<Vec<u8>>, ImageSizeT, ImageColourType) = match load_from_disk {
            false => {
                debug!("Reloading image from memory... at the spweed of a spwinting c-cat meow :3 (wait WTF!?!?)...");

                let image_data_mutex = self.image_data.lock().unwrap();

                let image_data = image_data_mutex
                    .as_ref()
                    .expect(
                        "Image has no image data loaded in memory so we \
                        cannot reload the image! This is a logic error, report this!"
                    );

                match image_data {
                    ImageData::Pixels(pixels) => pixels.clone(),
                    // images loaded into memory as bytes will never be called to reload as they will never have modifications.
                    ImageData::StaticBytes(_) => {
                        debug!(
                            "Image got reloaded but there's no reason to reload this image as it \
                                is a static type (StaticBytes) hence we will skip this image reload."
                        );
                        return Ok(());
                    },
                }
            },
            true => {
                debug!("Reloading image from disk...");

                // clear memory as we aren't going to use that any more.
                // *self.image_bytes.lock().unwrap() = None;

                let image_file = self.get_image_file()?;

                let mut image_buf_reader = BufReader::new(image_file);

                let mut decoded_image = self.decode_image(
                    image_processing_backend,
                    &mut image_buf_reader,
                    notifier
                )?;

                if let DecodedImage::Egui = decoded_image {
                    let mut buffer = Vec::new();
                    // TODO: handle error
                    image_buf_reader.read_to_end(&mut buffer).unwrap();

                    *self.image_data.lock().unwrap() = Some(
                        ImageData::StaticBytes(Arc::new(buffer))
                    );
        
                    return Ok(());
                }

                if !modifications.is_empty() {
                    decoded_image = self.modify_decoded_image(
                        modifications,
                        decoded_image,
                        notifier
                    )?;
                }

                let (pixels, image_size, image_colour_type) = self.decoded_image_to_pixels(
                    decoded_image
                )?;

                (Arc::new(pixels), image_size, image_colour_type)
            },
        };

        *self.current_modifications.lock().unwrap() = current_modifications;
        *self.image_data.lock().unwrap() = Some(ImageData::Pixels(arc_pixels));

        Ok(())
    }

    pub fn load_image(
        &mut self,
        notifier: &mut NotifierAPI,
        monitor_size: &MonitorSize,
        modifications: HashSet<ImageModifications>,
        image_processing_backend: &ImageProcessingBackend
    ) -> Result<()> {
        notifier.set_loading(Some("Opening file...".into()));
        debug!("Opening file into buf reader to prepare for reading...");

        let image_file = self.get_image_file()?;

        let mut image_buf_reader = BufReader::new(image_file); // apparently this is faster for larger files as 
        // it avoids loading files line by line hence less system calls to the disk. (EDIT: I'm defiantly noticing a speed difference)

        notifier.set_loading(Some("Decoding image...".into()));

        let mut decoded_image = self.decode_image(
            image_processing_backend, &mut image_buf_reader, notifier
        )?;

        if let DecodedImage::Egui = decoded_image {
            let mut buffer = Vec::new();
            // TODO: handle error
            image_buf_reader.read_to_end(&mut buffer).unwrap();

            *self.image_data.lock().unwrap() = Some(
                ImageData::StaticBytes(Arc::new(buffer))
            );

            return Ok(());
        }

        let current_modifications = modifications.clone();

        if !modifications.is_empty() {
            decoded_image = self.modify_decoded_image(
                modifications,
                decoded_image,
                notifier
            )?;
        }

        let image_pixels_result = self.decoded_image_to_pixels(decoded_image);

        match image_pixels_result {
            Ok((image_pixels, image_size, image_colour_type)) => {
                *self.current_modifications.lock().unwrap() = current_modifications;

                *self.image_data.lock().unwrap() = Some(
                    ImageData::Pixels((Arc::from(image_pixels), image_size, image_colour_type))
                );

                Ok(())
            },
            Err(error) => {
                let error = Error::FailedToApplyOptimizations(
                    Some(error.to_string()),
                    "Failed to decode and load image to apply modifications!".to_string()
                );

                // warn the user that modifications failed to apply.
                notifier.toasts.lock().unwrap()
                    .toast_and_log(error.into(), egui_notify::ToastLevel::Error);
    
                // load image without modifications
                // TODO: this needs to go when we move to "image_pixels" with 
                // #57 (https://github.com/cloudy-org/roseate/issues/57) and also 
                // when #58 (https://github.com/cloudy-org/roseate/issues/58) is completed.
                self.load_image(
                    notifier,
                    monitor_size,
                    HashSet::new(),
                    image_processing_backend
                )
            },
        }
    }

    fn get_image_file(&self) -> Result<File> {
        match File::open(self.image_path.as_ref()) {
            Ok(file) => Ok(file),
            Err(error) => {
                Err(
                    Error::FileNotFound(
                        Some(error.to_string()),
                        self.image_path.to_path_buf(),
                        "The file we're trying to load does not exist any more!
                        This might suggest that the image got deleted between the
                        time you opened it and roseate was ready to load it.".to_string()
                    )
                )
            },
        }
    }

    /// Check if modifications in both hash sets are the same by deep comparing them.
    fn are_modifications_the_same(&self, a: &HashSet<ImageModifications>, b: &HashSet<ImageModifications>) -> bool {
        if a.len() != b.len() {
            return false;
        }

        fn hash_modification(m: &ImageModifications) -> u64 {
            let mut hasher = DefaultHasher::new();

            match m {
                ImageModifications::Resize((width, height)) => {
                    width.hash(&mut hasher);
                    height.hash(&mut hasher);
                },
            }

            hasher.finish()
        }

        let a_hashes: HashSet<u64> = a.iter().map(hash_modification).collect();
        let b_hashes: HashSet<u64> = b.iter().map(hash_modification).collect();

        a_hashes == b_hashes
    }

    fn are_modifications_outside_memory_bounds(&self, modifications_to_apply: &HashSet<ImageModifications>) -> bool {
        let current_modifications = self.current_modifications.lock().unwrap();

        for current_modification in current_modifications.iter() {
            match (modifications_to_apply.get(&current_modification), current_modification) {
                (
                    Some(ImageModifications::Resize((width, height))),
                    ImageModifications::Resize((current_width, current_height))
                ) => {
                    // If this "if" statement evaluates to true this 
                    // means we are being asked to resize the image upwards 
                    // to data we do not have in memory, hence we cannot use what's 
                    // in memory / these modifications are outside the memory bounds.
                    if (width > current_width) | (height > current_height) {
                        return true;
                    }
                },
                (None, ImageModifications::Resize(_)) => {
                    return true;
                },
            }
        }

        return false;
    }

}

fn get_svg_image_size(path: &Path) -> ImageSizeT {
    let metadata = Metadata::parse_file(path).expect(
        "Failed to parse metadata of the svg file!"
    );

    let width = metadata.width().expect("Failed to get SVG width");
    let height = metadata.height().expect("Failed to get SVG height");

    // let display_info = get_primary_display_info();

    // let image_to_display_ratio = Vec2::new(width as f32, height as f32) /
    //    Vec2::new(display_info.width as f32, display_info.height as f32);

    // Temporary solution to give svg images a little bit higher quality.
    // ImageSize {
    //     width: (width * (1.0 + (1.0 - image_to_display_ratio.x)) as f64) as usize,
    //     height: (height * (1.0 + (1.0 - image_to_display_ratio.y)) as f64) as usize 
    // }

    // NOTE: Commented out the above lines as we are no longer using display-info crate.
    // Sadly this means svg images now will be even broken and worse image quality.
    // too bad... deal with it... *for now*

    (width as u32, height as u32)
}