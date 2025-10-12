use std::{collections::HashSet, hash::{DefaultHasher, Hash, Hasher}, path::Path, sync::{Arc, Mutex}, thread, time::{Duration, Instant}};

use cirrus_egui::v1::{notifier::Notifier, scheduler::Scheduler};
use eframe::egui::Context;
use egui::{TextureHandle, TextureOptions};
use rfd::FileDialog;
use log::{debug, info, warn};
use monitor_downsampling::get_monitor_downsampling_size;
use optimization::ImageOptimizations;

use crate::{error::{Error, Result}, image::{backends::ImageProcessingBackend, image::{Image, ImageSizeT}, image_data::{ImageColourType, ImageData}, image_formats::ImageFormat, modifications::ImageModifications}, monitor_size::MonitorSize, zoom_pan::ZoomPan};

mod dynamic_sampling;

pub mod optimization;
pub mod monitor_downsampling;

/// Struct that handles all the image loading logic in a thread safe 
/// manner to allow features such as background image loading / lazy loading.
/// 
/// ImageHandler struct is a ui facing, Image struct is low-level stuff.
pub struct ImageHandler {
    pub image: Option<Image>,
    pub image_loaded: bool,
    image_loading: bool,
    image_loaded_arc: Arc<Mutex<bool>>,
    egui_image_texture: Option<TextureHandle>,
    pub image_optimizations: HashSet<ImageOptimizations>,
    dynamic_sample_schedule: Option<Scheduler>,
    last_zoom_factor: f32,
    dynamic_sampling_new_resolution: ImageSizeT,
    dynamic_sampling_old_resolution: ImageSizeT,
    accumulated_zoom_factor_change: f32,
    forget_last_image_bytes_arc: Arc<Mutex<bool>>,
    monitor_downsampling_required: bool,
}

impl ImageHandler {
    pub fn new() -> Self {
        Self {
            image: None,
            image_loaded: false,
            image_loaded_arc: Arc::new(Mutex::new(false)),
            image_loading: false,
            image_optimizations: HashSet::new(),
            dynamic_sample_schedule: None,
            egui_image_texture: None,
            last_zoom_factor: 1.0,
            dynamic_sampling_new_resolution: (0, 0),
            dynamic_sampling_old_resolution: (0, 0),
            accumulated_zoom_factor_change: 0.0,
            forget_last_image_bytes_arc: Arc::new(Mutex::new(false)),
            monitor_downsampling_required: false
        }
    }

    pub fn init_image(&mut self, image_path: &Path, image_optimizations: Vec<ImageOptimizations>) -> Result<()> {
        let image = Image::from_path(image_path)?;

        self.image_optimizations = HashSet::from_iter(image_optimizations);
        self.image = Some(image);

        Ok(())
    }

    pub fn select_image(&mut self, image_optimizations: Vec<ImageOptimizations>) -> Result<()> {
        let image_path = FileDialog::new()
            .add_filter("images", &["png", "jpeg", "jpg", "webp", "gif", "svg"])
            .pick_file();

        match image_path {
            Some(path) => {
                if !path.exists() {
                    return Err(
                        Error::FileNotFound(
                            None,
                            path,
                            "The file picked in the file selector does not exist!".to_string()
                        )
                    )
                }

                self.init_image(&path, image_optimizations)?;

                Ok(())
            },
            None => Err(
                Error::NoFileSelected(None)
            )
        }
    }

    pub fn update(
        &mut self,
        ctx: &Context,
        zoom_factor: &f32,
        is_panning: bool,
        monitor_size: &MonitorSize,
        notifier: &mut Notifier,
        backend: ImageProcessingBackend
    ) {
        // I use an update function to keep the public 
        // fields update to date with their Arc<Mutex<T>> twins
        // and also now to perform dynamic downsampling.

        if let Ok(value) = self.image_loaded_arc.try_lock() {
            self.image_loaded = value.clone(); // cloning here shouldn't be too expensive
            self.image_loading = false; // set that bitch back to false yeah
        }

        if self.image_loaded {
            if *self.forget_last_image_bytes_arc.lock().unwrap() {
                notifier.set_loading(Some("Releasing some memory...".into()));
                debug!("Releasing last image bytes from egui's memory...");
                ctx.forget_all_images();

                notifier.unset_loading();
                self.egui_image_texture = None;
                *self.forget_last_image_bytes_arc.lock().unwrap() = false;
            }
        }

        self.dynamic_sampling_update(zoom_factor, monitor_size);

        if let Some(schedule) = &mut self.dynamic_sample_schedule {
            // TODO: if we are still panning once we have stopped 
            // defer some addition seconds to the dynamic_sample_schedule.
            if !is_panning {
                if schedule.update().is_some() {
                    if self.dynamic_sampling_new_resolution == self.dynamic_sampling_old_resolution {
                        debug!(
                            "Will not schedule this dynamic sample ({:?} -> {:?}) \
                            as it's going to sample to the same resolution!",
                            self.dynamic_sampling_old_resolution,
                            self.dynamic_sampling_new_resolution
                        );
                        return;
                    }

                    self.load_image(
                        true,
                        notifier,
                        monitor_size,
                        backend
                    );
                }
            }
        }
    }

    /// Handles loading the image in a background thread or on the main thread. 
    /// Set `lazy_load` to `true` if you want the image to be loaded in the background on a separate thread.
    /// 
    /// Setting `lazy_load` to `false` **will block the main thread** until the image is loaded.
    pub fn load_image(&mut self, lazy_load: bool, notifier: &mut Notifier, monitor_size: &MonitorSize, backend: ImageProcessingBackend) {
        if self.image_loading {
            warn!("Not loading image as one is already being loaded!");
            return;
        }

        self.image_loading = true;

        notifier.set_loading(
            Some("Preparing to load image...".into())
        );

        let mut image = self.image.clone().expect(
            "You must run 'ImageHandler.init_image()' before using 'ImageHandler.load_image()'!"
        );

        notifier.set_loading(
            Some("Gathering necessary image modifications...".into())
        );

        let image_modifications = self.get_image_modifications(
            &monitor_size
        );

        let image_modifications_display = format!("{:?}", image_modifications);

        // Our svg implementation is very experimental. 
        // Also broken! https://github.com/cloudy-org/roseate/issues/66 
        // Let's warn the user.
        if ImageFormat::Svg == image.image_format {
            notifier.toast(
                "SVG files are experimental and broken! \
                Expect many bugs, inconstancies and performance / memory issues.",
                egui_notify::ToastLevel::Warning,
                |toast| {
                    toast.duration(Some(Duration::from_secs(8)));
                }
            );
        }

        let image_loaded_arc = self.image_loaded_arc.clone();
        let forget_last_image_bytes_arc = self.forget_last_image_bytes_arc.clone();
        let mut notifier_arc = notifier.clone();
        let monitor_size_arc = monitor_size.clone();

        let loading_logic = move || {
            let now = Instant::now();
            let mut hasher = DefaultHasher::new();

            let result = match *image_loaded_arc.lock().unwrap() {
                true => {
                    notifier_arc.set_loading(Some("Reloading image...".into()));

                    let result = image.reload_image(
                        &mut notifier_arc,
                        image_modifications,
                        &backend
                    );

                    debug!(
                        "Image reloaded in '{}' seconds using '{}' backend.",
                        now.elapsed().as_secs_f32(),
                        backend
                    );

                    result
                },
                false => {
                    notifier_arc.set_loading(Some("Loading image...".into()));

                    let result = image.load_image(
                        &mut notifier_arc,
                        &monitor_size_arc,
                        image_modifications,
                        &backend
                    );

                    info!(
                        "Image loaded in '{}' seconds using '{}' backend.", 
                        now.elapsed().as_secs_f32(), backend
                    );

                    result
                }
            };

            match result {
                Ok(()) => {
                    *image_loaded_arc.lock().unwrap() = true;

                    *forget_last_image_bytes_arc.lock().unwrap() = true;

                    image.hash(&mut hasher);
                    debug!("Image data hash: {}", hasher.finish());
                    debug!("Image current modifications: {}", image_modifications_display);
                },
                Err(error) => {
                    notifier_arc.toast(
                        Box::new(error),
                        egui_notify::ToastLevel::Error,
                        |toast| {
                            toast.duration(Some(Duration::from_secs(10)));
                        }
                    );
                },
            }

            notifier_arc.unset_loading();
        };

        if lazy_load {
            debug!("Lazy loading image (in a thread)...");
            thread::spawn(loading_logic);
        } else {
            debug!("Loading image in main thread...");
            loading_logic();
        }
    }

    pub fn get_egui_image(&mut self, ctx: &egui::Context) -> egui::Image {
        assert!(
            self.image_loaded,
            "'ImageHandler::get_egui_image()' should never be called if 'self.image_loaded' is true!"
        );

        let image = self.image.as_ref().unwrap();

        let mut hasher = DefaultHasher::new();
        image.hash(&mut hasher);

        let image_hash = hasher.finish();

        // we can unwrap Option<T> here as if 
        // "self.image_handler.image_loaded" is true image data should exist.
        match image.image_data.lock().unwrap().as_ref()
            .expect("Image data was not present! This is a logic error on our side, please report it.") {
            ImageData::Pixels((pixels, (width, height), image_colour_type)) => {
                let texture = match &self.egui_image_texture {
                    Some(texture) => texture,
                    None => {
                        self.egui_image_texture = Some(
                            ctx.load_texture(
                                "image",
                                match image_colour_type {
                                    ImageColourType::Grey | ImageColourType::GreyAlpha => {
                                        debug!("Rendering image as grey scale egui texture...");
                                        egui::ColorImage::from_gray(
                                            [*width as usize, *height as usize],
                                            pixels.as_slice()
                                        )
                                    },
                                    ImageColourType::RGB => {
                                        debug!("Rendering image as rgb egui texture...");
                                        egui::ColorImage::from_rgb(
                                            [*width as usize, *height as usize],
                                            pixels.as_slice()
                                        )
                                    },
                                    ImageColourType::RGBA => {
                                        debug!("Rendering image as rgba egui texture...");
                                        egui::ColorImage::from_rgba_unmultiplied(
                                            [*width as usize, *height as usize],
                                            pixels.as_slice()
                                        )
                                    },
                                },
                                TextureOptions::default()
                            )
                        );

                        &self.egui_image_texture.as_ref().unwrap()
                    },
                };

                egui::Image::from_texture(texture)
            },
            ImageData::StaticBytes(bytes) => {
                egui::Image::from_bytes(
                    format!("bytes://{}.{:#}", image_hash, image.image_format),
                    bytes.to_vec() // TODO: I think this duplicates memory so 
                    // this will need to be analysed and changed.
                )
            },
        }
    }

    /// Method that handles choosing which type of modifications 
    /// should be done to the image at this time. It decides that on a number of various factors, 
    /// like image optimizations applied by the user, monitor size, zoom factor and etc.
    fn get_image_modifications(&mut self, monitor_size: &MonitorSize) -> HashSet<ImageModifications> {
        let mut image_modifications = HashSet::new();

        let image = self.image.as_ref();

        if let Some(image) = image {
            // SVG and GIFs should not get image modifications.
            if let ImageFormat::Svg | ImageFormat::Gif = image.image_format {
                return image_modifications;
            }

            // the reason why we don't just loop over self.image_optimizations 
            // is because I need to make absolute sure I'm doing these checks in this exact order.

            if let Some(ImageOptimizations::MonitorDownsampling(marginal_allowance)) = self.image_optimizations.get(
                &ImageOptimizations::MonitorDownsampling(u32::default())
            ) {
                let (width, height) = get_monitor_downsampling_size(
                    marginal_allowance, monitor_size
                );

                // If the image is a lot bigger than the user's 
                // monitor then apply monitor downsample, if not we shouldn't.
                if image.image_size.0 > width && image.image_size.1 > height {
                    self.monitor_downsampling_required = true;

                    debug!(
                        "Image is significantly bigger than system's \
                        display monitor so monitor downsampling will be applied..."
                    );

                    let image_size = (image.image_size.0, image.image_size.1);

                    debug!(
                        "Image Size: {} x {}", image_size.0, image_size.1
                    );

                    let (monitor_width, monitor_height) = monitor_size.get();

                    debug!(
                        "Display (Monitor) Size: {} x {}", monitor_width, monitor_height
                    );

                    debug!(
                        "Display + Monitor Downsample Marginal Allowance ({}): {} x {}",
                        marginal_allowance, width, height
                    );

                    image_modifications.replace(ImageModifications::Resize((width, height)));
                }
            }

            // TODO: handle up and down dyn sampling options.
            // NOTE: I think I might just add "down" as a bool tbh, you'll never want 
            // upsampling to be disabled if you choose to enable dyn sampling in the first place.
            if let Some(ImageOptimizations::DynamicSampling(up, down)) = self.image_optimizations.get(
                &ImageOptimizations::DynamicSampling(bool::default(), bool::default())
            ) {
                let new_resolution = self.dynamic_sampling_new_resolution;
                let old_resolution = self.dynamic_sampling_old_resolution;

                if !(new_resolution == old_resolution) {
                    debug!(
                        "User zoomed far enough into downsampled image, \
                        dynamic sampling will be performed... \n\t({:?} -> {:?})",
                        old_resolution, new_resolution
                    );

                    if !(new_resolution.0 == image.image_size.0 && new_resolution.1 == image.image_size.1) {
                        image_modifications.replace(
                            ImageModifications::Resize(new_resolution)
                        );

                        self.dynamic_sampling_old_resolution = new_resolution;
                    } else {
                        debug!(
                            "Not applying resize mod for dynamic sampling as \
                            dynamic sampling is requesting the full resolution already!"
                        );

                        image_modifications.remove(
                            &ImageModifications::Resize(new_resolution)
                        );

                        self.dynamic_sampling_old_resolution = (
                            image.image_size.0,
                            image.image_size.1
                        );
                    }
                }
            }
        };

        image_modifications
    }
}