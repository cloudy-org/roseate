use std::{collections::HashSet, hash::{DefaultHasher, Hash, Hasher}, sync::{Arc, Mutex}, thread, time::{Duration, Instant}};

use cirrus_egui::{notifier::Notifier, scheduler::Scheduler};
use cirrus_soft_binds::egui::BoxedEguiInputReaderFunc;
use eframe::egui::Ui;
use egui_notify::ToastLevel;
use log::{debug, info, warn};
use roseate_core::{decoded_image::ImageSize, format::ImageFormat, modifications::{ImageModification, ImageModifications}};

use crate::{image::{Image, backend::DefaultDecodingBackend}, image_loader::{uploaded_image::UploadedImage, optimization::ImageOptimizations}, image_selector::ImageSelector, monitor_size::MonitorSize};

pub struct ImageLoader {
    pub image_loading: bool,
    pub image_optimizations: ImageOptimizations,

    pub(super) dynamic_sample_schedule: Option<Scheduler>,
    pub(super) last_zoom_factor: f32,
    pub(super) dynamic_sampling_new_resolution: ImageSize,
    pub(super) dynamic_sampling_old_resolution: ImageSize,
    pub(super) accumulated_zoom_factor_change: f32,
    pub(super) monitor_downsampling_required: bool,

    pub(super) uploaded_image: Option<UploadedImage>,
    pub(super) load_image_to_gpu: Arc<Mutex<bool>>,

    new_image_experimental_warning_shown: bool,
}

impl ImageLoader {
    pub fn new(image_optimizations: ImageOptimizations) -> Self {
        Self {
            image_loading: false,

            image_optimizations,

            dynamic_sample_schedule: None,
            last_zoom_factor: 1.0,
            dynamic_sampling_new_resolution: (0, 0),
            dynamic_sampling_old_resolution: (0, 0),
            accumulated_zoom_factor_change: 0.0,
            monitor_downsampling_required: false,

            uploaded_image: None,
            load_image_to_gpu: Arc::new(Mutex::new(false)),

            new_image_experimental_warning_shown: false
        }
    }

    pub fn handle_input(
        &mut self,
        ui: &Ui,
        image_selector: &mut ImageSelector,
        monitor_size: &MonitorSize,
        backend: DefaultDecodingBackend,
        notifier: &mut Notifier,

        open_image_input_reader: &mut BoxedEguiInputReaderFunc
    ) {
        if ui.input(open_image_input_reader) {
            if image_selector.get_image().is_some() && !self.new_image_experimental_warning_shown {
                notifier.toast(
                    "Loading a new image is currently experimental, expect bugs.",
                    ToastLevel::Warning,
                    |toast| {
                        toast.duration(Duration::from_secs(40));
                    }
                );

                self.new_image_experimental_warning_shown = true;
            }

            if let Err(error) = image_selector.select_image_from_file_explorer() {
                notifier.toast(
                    Box::new(error),
                    ToastLevel::Error,
                    |toast| {
                        toast.duration(Duration::from_secs(5));
                    }
                );

                return;
            }

            if let Some(image) = image_selector.get_mutable_image() {
                // reset the image loader (set all values back to default)
                self.dynamic_sample_schedule = None;
                self.last_zoom_factor = 1.0;
                self.dynamic_sampling_new_resolution = ImageSize::default();
                self.dynamic_sampling_old_resolution = ImageSize::default();
                self.accumulated_zoom_factor_change = 0.0;
                self.monitor_downsampling_required = false;
                // self.uploaded_image = None;
                // self.load_image_to_gpu = Arc::new(Mutex::new(false));

                self.load(
                    image,
                    true,
                    backend,
                    monitor_size,
                    notifier,
                );
            }
        }
    }

    pub fn load(
        &mut self,
        image: &mut Image,
        lazy_load: bool,
        backend: DefaultDecodingBackend,
        monitor_size: &MonitorSize,
        notifier: &mut Notifier
    ) {
        if self.image_loading { // would we ever even hit this?
            warn!("Not loading image as one is already being loaded!");
            return;
        }

        let mut image_modifications = self.get_image_modifications(
            &image.size,
            monitor_size,
        );

        let image_modifications_debug = format!("{:?}", image_modifications);

        let use_experimental_multi_threaded_downsampling = match &self.image_optimizations.multi_threaded_sampling {
            Some(multi_threaded_sampling) => {
                Self::snatch_resize_modification_and_get_size(&mut image_modifications)
                    .and_then(|target_size| Some((target_size, multi_threaded_sampling.number_of_threads)))
            },
            None => None,
        };

        self.image_loading = true;

        notifier.set_loading(
            Some("Gathering necessary image modifications...")
        );

        // Our svg implementation is very experimental. 
        // Also broken! https://github.com/cloudy-org/roseate/issues/66 
        // Let's warn the user.
        if ImageFormat::Svg == image.format {
            notifier.toast(
                "SVG files are experimental and broken! \
                Expect many bugs, inconstancies and performance / memory issues.",
                egui_notify::ToastLevel::Warning,
                |toast| {
                    toast.duration(Some(Duration::from_secs(8)));
                }
            );
        }

        // let image_loaded_arc = self.image_loaded_arc.clone();
        let mut image_clone = image.clone();
        let mut notifier_clone = notifier.clone();

        let load_image_to_gpu_arc = self.load_image_to_gpu.clone();

        let reload_image = match &self.uploaded_image {
            Some(uploaded_image) => {
                let mut hasher = DefaultHasher::new();
                image.hash(&mut hasher);

                // if this is not the same image perform a full load instead of a reload.
                uploaded_image.image_hash == hasher.finish()
            },
            None => false,
        };

        let loading_logic = move || {
            let now = Instant::now();

            let result = match reload_image {
                true => {
                    notifier_clone.set_loading(Some("Reloading image..."));

                    let result = image_clone.load(
                        image_modifications,
                        &backend,
                        true,
                        &mut notifier_clone
                    );

                    debug!(
                        "Image reloaded in '{}' seconds using '{}' backend.",
                        now.elapsed().as_secs_f32(),
                        backend
                    );

                    result
                },
                false => {
                    notifier_clone.set_loading(Some("Loading image..."));

                    let result = image_clone.load(
                        image_modifications,
                        &backend,
                        false,
                        &mut notifier_clone,
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
                    if let Some((target_size, number_of_threads)) = use_experimental_multi_threaded_downsampling {
                        notifier_clone.set_loading(Some("Performing fast multi-threaded downsampling..."));
                        Self::perform_multi_threaded_downsample(
                            target_size,
                            &mut image_clone,
                            number_of_threads
                        );
                        notifier_clone.unset_loading();
                    }

                    *load_image_to_gpu_arc.lock().unwrap() = true;

                    debug!("Image debug: {:?}", image_clone);
                    debug!("Image modifications debug: {}", image_modifications_debug);
                },
                Err(error) => {
                    notifier_clone.toast(
                        Box::new(error),
                        egui_notify::ToastLevel::Error,
                        |toast| {
                            toast.duration(Some(Duration::from_secs(10)));
                        }
                    );

                    // TODO: we need to set image loading back to false on 
                    // decoder errors, otherwise the home menu will be frozen.

                    // self.image_loading = false;
                },
            }

            notifier_clone.unset_loading();
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
    fn get_image_modifications(&mut self, image_size: &ImageSize, monitor_size: &MonitorSize) -> ImageModifications {
        let mut image_modifications = HashSet::new();

        if let Some(monitor_downsampling) = &self.image_optimizations.monitor_downsampling {
            let (max_width, max_height) = monitor_downsampling.get_size_relative_to_monitor(&monitor_size);
            let scale = (max_width as f32 / image_size.0 as f32).min(max_height as f32 / image_size.1 as f32);

            // If the image is a lot bigger than the user's 
            // monitor then apply monitor downsample, if not we shouldn't.
            if scale < 1.0 {
                self.monitor_downsampling_required = true;

                let (width, height) = (image_size.0 as f32 * scale, image_size.1 as f32 * scale);

                debug!(
                    "Image is significantly bigger than system's \
                    display monitor so monitor downsampling will be applied..."
                );

                debug!(
                    "Image Size: {} x {}", image_size.0, image_size.1
                );

                let (monitor_width, monitor_height) = monitor_size.get();

                debug!(
                    "Display (Monitor) Size: {} x {}", monitor_width, monitor_height
                );

                debug!(
                    "Display + Monitor Downsample Marginal Allowance ({}): {} x {}",
                    monitor_downsampling.marginal_allowance, width, height
                );

                image_modifications.replace(
                    ImageModification::Resize(width.round() as u32, height.round() as u32)
                );
            }
        }

        if let Some(dynamic_sampling) = &self.image_optimizations.dynamic_sampling {
            // TODO: handle up and down dyn sampling options.
            // NOTE: I think I might just add "down" as a bool tbh, you'll never want 
            // upsampling to be disabled if you choose to enable dyn sampling in the first place.
            let (up, down) = (dynamic_sampling.up, dynamic_sampling.down);

            let new_resolution = self.dynamic_sampling_new_resolution;
            let old_resolution = self.dynamic_sampling_old_resolution;

            if !(new_resolution == old_resolution) {
                debug!(
                    "User zoomed far enough into downsampled image, \
                    dynamic sampling will be performed... \n\t({:?} -> {:?})",
                    old_resolution, new_resolution
                );

                if !(new_resolution.0 == image_size.0 && new_resolution.1 == image_size.1) {
                    image_modifications.replace(
                        ImageModification::Resize(new_resolution.0, new_resolution.1)
                    );

                    self.dynamic_sampling_old_resolution = new_resolution;
                } else {
                    debug!(
                        "Not applying resize mod for dynamic sampling as \
                        dynamic sampling is requesting the full resolution already!"
                    );

                    image_modifications.remove(
                        &ImageModification::Resize(new_resolution.0, new_resolution.1)
                    );

                    self.dynamic_sampling_old_resolution = (
                        image_size.0,
                        image_size.1
                    );
                }
            }
        }

        image_modifications
    }
}