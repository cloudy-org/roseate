use std::{collections::HashSet, hash::{DefaultHasher, Hash, Hasher}, path::Path, sync::{Arc, Mutex}, thread, time::{Duration, Instant}};

use eframe::egui::{load::BytesLoader, Context};
use rfd::FileDialog;
use log::{debug, info, warn};
use monitor_downsampling::get_monitor_downsampling_size;
use optimization::ImageOptimizations;

use crate::{error::{Error, Result}, image::{backends::ImageProcessingBackend, image::{Image, ImageSizeT}, modifications::ImageModifications}, monitor_size::MonitorSize, notifier::NotifierAPI, scheduler::Scheduler, zoom_pan::ZoomPan};

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
    pub image_optimizations: HashSet<ImageOptimizations>,
    dynamic_sample_schedule: Option<Scheduler<ImageSizeT>>,
    last_zoom_factor: f32,
    dynamic_sampling_new_resolution: ImageSizeT,
    dynamic_sampling_old_resolution: ImageSizeT,
    accumulated_zoom_factor_change: f32,
    forget_last_image_bytes_arc: Arc<Mutex<bool>>
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
            last_zoom_factor: 1.0,
            dynamic_sampling_new_resolution: (0, 0),
            dynamic_sampling_old_resolution: (0, 0),
            accumulated_zoom_factor_change: 0.0,
            forget_last_image_bytes_arc: Arc::new(Mutex::new(false)),
        }
    }

    pub fn init_image(&mut self, image_path: &Path) -> Result<()> {
        let image = Image::from_path(image_path)?;

        // TODO: pass optimizations into this function instead 
        // or have the developer manipulate image_optimizations directly.
        // TODO: also maybe just have the image optimization set when ImageHandler is initialized.
        self.image_optimizations = HashSet::from_iter(
            vec![
                ImageOptimizations::MonitorDownsampling(130),
                ImageOptimizations::DynamicSampling(true, true)
            ].iter().cloned()
        );

        self.image = Some(image);

        Ok(())
    }

    pub fn select_image(&mut self, monitor_size: &MonitorSize) -> Result<()> {
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

                self.init_image(&path)?;

                Ok(())
            },
            None => Err(Error::NoFileSelected(None))
        }
    }

    pub fn update(
        &mut self,
        ctx: &Context,
        zoom_pan: &ZoomPan,
        monitor_size: &MonitorSize,
        notifier: &mut NotifierAPI,
        use_experimental_backend: bool
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
                debug!("Releasing last image bytes from egui's memory...");
                ctx.forget_all_images();
                *self.forget_last_image_bytes_arc.lock().unwrap() = false;
            }
        }

        self.dynamic_sampling_update(zoom_pan, monitor_size);

        if let Some(schedule) = &mut self.dynamic_sample_schedule {
            if !zoom_pan.is_panning {
                if let Some(new_resolution) = schedule.update() {
                    self.dynamic_sampling_new_resolution = new_resolution;

                    self.load_image(
                        true,
                        true,
                        notifier,
                        monitor_size,
                        use_experimental_backend
                    );
                }
            }
        }
    }

    // TODO: (28/03/2025) ImageHandler::load_image should decide whether we image.reload or image.load.

    /// Handles loading the image in a background thread or on the main thread. 
    /// Set `lazy_load` to `true` if you want the image to be loaded in the background on a separate thread.
    /// 
    /// Setting `lazy_load` to `false` **will block the main thread** until the image is loaded.
    pub fn load_image(&mut self, lazy_load: bool, reload: bool, notifier: &mut NotifierAPI, monitor_size: &MonitorSize, use_experimental_backend: bool) {
        if self.image_loading {
            warn!("Not loading image as one is already being loaded!");
            return;
        }

        self.image_loading = true;

        notifier.set_loading_and_log(
            Some("Preparing to load image...".into())
        );

        let mut image = self.image.clone().expect(
            "You must run 'ImageHandler.init_image()' before using 'ImageHandler.load_image()'!"
        );

        notifier.set_loading_and_log(
            Some("Gathering necessary image modifications...".into())
        );

        let mut image_modifications = self.get_image_modifications(
            &image, &monitor_size
        );

        // Our svg implementation is very experimental. Let's warn the user.
        if image.image_path.extension().unwrap_or_default() == "svg" {
            // TODO: Allow svg enum in image.image_format.
            notifier.toasts.lock().unwrap()
                .toast_and_log(
                    "SVG files are experimental! \
                    Expect many bugs, inconstancies and performance issues.".into(),
                egui_notify::ToastLevel::Warning
                )
                .duration(Some(Duration::from_secs(8)));

            // SVGs cannot be loaded with modifications at 
            // the moment or else image.load_image() will panic.
            image_modifications.clear();
        }

        let image_loaded_arc = self.image_loaded_arc.clone();
        let forget_last_image_bytes_arc = self.forget_last_image_bytes_arc.clone();
        let mut notifier_arc = notifier.clone();
        let monitor_size_arc = monitor_size.clone();

        let loading_logic = move || {
            let backend = match use_experimental_backend {
                true => ImageProcessingBackend::Roseate,
                false => ImageProcessingBackend::ImageRS
            };

            let now = Instant::now();

            let result = match reload {
                true => {
                    notifier_arc.set_loading_and_log(Some("Reloading image...".into()));

                    // TODO: use low level reload image method instead when that is implemented
                    let result = image.load_image(
                        &mut notifier_arc,
                        &monitor_size_arc,
                        image_modifications,
                        &backend
                    );

                    debug!(
                        "Image reloaded in '{}' seconds using '{}' backend.", 
                        now.elapsed().as_secs_f32(), backend
                    );

                    result
                },
                false => {
                    notifier_arc.set_loading_and_log(Some("Loading image...".into()));
        
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

            if let Err(error) = result {
                notifier_arc.toasts.lock().unwrap()
                    .toast_and_log(error.into(), egui_notify::ToastLevel::Error)
                    .duration(Some(Duration::from_secs(10)));
            }

            notifier_arc.unset_loading();
            *image_loaded_arc.lock().unwrap() = true;

            *forget_last_image_bytes_arc.lock().unwrap() = true;
        };

        if lazy_load {
            debug!("Lazy loading image (in a thread)...");
            thread::spawn(loading_logic);
        } else {
            debug!("Loading image in main thread...");
            loading_logic();
        }
    }

    /// Method that handles choosing which type of modifications 
    /// should be done to the image at this time. It decides that on a number of various factors, 
    /// like image optimizations applied by the user, monitor size, zoom factor and etc.
    fn get_image_modifications(&mut self, image: &Image, monitor_size: &MonitorSize) -> HashSet<ImageModifications> {
        let mut image_modifications = HashSet::new();

        // the reason why we don't just loop over self.image_optimizations 
        // is because I need to make absolute sure I'm doing these checks in this exact order.

        if let Some(ImageOptimizations::MonitorDownsampling(marginal_allowance)) = self.has_optimization(
            &ImageOptimizations::MonitorDownsampling(u32::default())
        ) {
            let (monitor_width, monitor_height) = monitor_size.get();

            // If the image is a lot bigger than the user's 
            // monitor then apply monitor downsample, if not we shouldn't.
            if image.image_size.width as u32 > monitor_width as u32 && image.image_size.height as u32 > monitor_height as u32 {
                debug!(
                    "Image is significantly bigger than system's \
                    display monitor so monitor downsampling will be applied..."
                );

                let image_size = (image.image_size.width, image.image_size.height);

                debug!(
                    "Image Size: {} x {}", image_size.0, image_size.1
                );

                let (monitor_width, monitor_height) = monitor_size.get();

                debug!(
                    "Display (Monitor) Size: {} x {}", monitor_width, monitor_height
                );

                let (width, height) = get_monitor_downsampling_size(
                    *marginal_allowance, monitor_size
                );

                debug!(
                    "Display + Monitor Downsample Marginal Allowance ({}): {} x {}",
                    marginal_allowance, width, height
                );

                image_modifications.replace(ImageModifications::Resize((width, height)));
            }
        }

        if let Some(ImageOptimizations::DynamicSampling(up, down)) = self.has_optimization(
            &ImageOptimizations::DynamicSampling(bool::default(), bool::default())
        ) {
            let new_resolution = self.dynamic_sampling_new_resolution;

            if !(new_resolution == self.dynamic_sampling_old_resolution) {
                debug!(
                    "User zoomed far enough into downsampled image, \
                    dynamic sampling will be performed ({:?} -> {:?})...",
                    self.dynamic_sampling_old_resolution,
                    self.dynamic_sampling_new_resolution
                );

                if !(new_resolution.0 == image.image_size.width as u32 && new_resolution.1 == image.image_size.height as u32) {
                    image_modifications.replace(
                        ImageModifications::Resize(new_resolution.clone())
                    );

                    self.dynamic_sampling_old_resolution = new_resolution;
                } else {
                    debug!(
                        "Not applying resize mod for dynamic sampling as \
                        dynamic sampling is requesting the full resolution already!"
                    );

                    image_modifications.remove(
                        &ImageModifications::Resize(new_resolution.clone())
                    );

                    self.dynamic_sampling_old_resolution = (
                        image.image_size.width as u32,
                        image.image_size.height as u32
                    );
                }
            }
        }

        image_modifications
    }

    /// Checks if the image has this TYPE of optimization applied, not the exact 
    /// optimization itself. Then it returns a reference to the exact optimization found.
    pub fn has_optimization(&self, optimization: &ImageOptimizations) -> Option<&ImageOptimizations> {
        for applied_optimization in self.image_optimizations.iter() {
            if applied_optimization.id() == optimization.id() {
                return Some(applied_optimization);
            }
        }

        return None;
    }
}