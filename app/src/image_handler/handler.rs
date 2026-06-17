use std::{collections::HashSet, hash::{DefaultHasher, Hash, Hasher}, sync::{Arc, Mutex}, thread, time::{Duration, Instant}};

use cirrus_egui::{notifier::Notifier, scheduler::Scheduler};
use cirrus_soft_binds::egui::BoxedEguiInputReaderFunc;
use eframe::egui::{Context, TextureFilter, TextureOptions, TextureWrapMode, Ui};
use egui_notify::ToastLevel;
use log::{debug, info, warn};
use roseate_core::{colour_type::ImageColourType, decoded_image::ImageSize, format::ImageFormat, modifications::{ImageModification, ImageModifications}};

use crate::{image::{Image, backend::DecodingBackend}, image_handler::{loaded_image::LoadedImage, optimization::ImageOptimizations, image_resource::ImageResource}, image_selector::ImageSelector, monitor_size::MonitorSize};

pub struct ImageHandler {
    pub loaded_image: Option<LoadedImage>,

    pub image_loading: bool,

    pub image_optimizations: ImageOptimizations,
    pub(super) dynamic_sample_schedule: Option<Scheduler>,
    pub(super) last_zoom_factor: f32,
    pub(super) dynamic_sampling_new_resolution: ImageSize,
    pub(super) dynamic_sampling_old_resolution: ImageSize,
    pub(super) accumulated_zoom_factor_change: f32,
    pub(super) monitor_downsampling_required: bool,
    pub(super) load_image_texture: Arc<Mutex<bool>>,
}

impl ImageHandler {
    pub fn new(image_optimizations: ImageOptimizations) -> Self {
        Self {
            loaded_image: None,

            image_loading: false,
            
            image_optimizations,

            dynamic_sample_schedule: None,
            last_zoom_factor: 1.0,
            dynamic_sampling_new_resolution: (0, 0),
            dynamic_sampling_old_resolution: (0, 0),
            accumulated_zoom_factor_change: 0.0,
            monitor_downsampling_required: false,

            load_image_texture: Arc::new(Mutex::new(false))
        }
    }

    pub fn handle_input(
        &mut self,
        ui: &Ui,
        image_selector: &mut ImageSelector,
        monitor_size: &MonitorSize,
        backend: DecodingBackend,
        notifier: &mut Notifier,

        open_image_input_reader: &mut BoxedEguiInputReaderFunc
    ) {
        if ui.input(open_image_input_reader) {
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
                self.load_image(
                    image,
                    true,
                    backend,
                    monitor_size,
                    notifier,
                );
            }
        }
    }

    pub fn update(
        &mut self,
        ctx: &Context,
        zoom_factor: &f32,
        is_panning: bool,
        image_selector: &mut ImageSelector,
        monitor_size: &MonitorSize,
        backend: DecodingBackend,
        notifier: &mut Notifier,
    ) {
        self.load_resource_update(ctx, image_selector, notifier);
        self.dynamic_sampling_update(zoom_factor, image_selector, monitor_size);

        if let Some(image) = image_selector.get_mutable_image() {

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
                            image,
                            true,
                            backend,
                            monitor_size,
                            notifier,
                        );
                    }
                }
            }
        }
    }

    pub fn load_image(
        &mut self,
        image: &mut Image,
        lazy_load: bool,
        backend: DecodingBackend,
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

        let load_image_texture_arc = self.load_image_texture.clone();

        let reload_image = match &self.loaded_image {
            Some(loaded_image) => {
                let mut hasher = DefaultHasher::new();
                image.hash(&mut hasher);

                // if this is not the same image perform a full load instead of a reload.
                loaded_image.image_hash == hasher.finish()
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

                    *load_image_texture_arc.lock().unwrap() = true;

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

    fn load_resource_update(&mut self, ctx: &Context, image_selector: &ImageSelector, notifier: &mut Notifier) {
        if let Some(image) = image_selector.get_image() {
            let reload_texture = match self.load_image_texture.try_lock() {
                Ok(load_image_texture_mutex) => *load_image_texture_mutex,
                Err(_) => false,
            };

            if reload_texture == false {
                return;
            }

            let can_free_memory_or_consume = self.image_optimizations.consume_pixels_during_gpu_upload;

            if let Some(decoded_image) = image.decoded.lock().unwrap().as_mut() {
                notifier.set_loading(Some("Converting image to texture to be uploaded to the GPU..."));

                let texture_options = TextureOptions {
                    magnification: TextureFilter::Linear,
                    minification: TextureFilter::Linear,
                    wrap_mode: TextureWrapMode::ClampToEdge,
                    mipmap_mode: None,
                };

                let is_rgba = matches!(
                    decoded_image.info.colour_type,
                    ImageColourType::Rgba8 | ImageColourType::Rgba16 | ImageColourType::Rgba32F
                );

                self.loaded_image = Some(
                    LoadedImage {
                        resource: match can_free_memory_or_consume && is_rgba {
                            true => ImageResource::from_rgba_decoded_image_zero_copy(ctx, decoded_image, texture_options),
                            false => ImageResource::from_decoded_image(ctx, &decoded_image, texture_options),
                        },
                        image_info: decoded_image.info.clone(),
                        image_hash: {
                            let mut hasher = DefaultHasher::new();

                            image.hash(&mut hasher);

                            hasher.finish()
                        }
                    }
                );

                // Texture handle doesn't need forgetting like egui::Image 
                // as it's smart enough to free itself from memory.

                ctx.forget_all_images(); // we want to free the rose image in 
                // image selection menu and all other potential images from memory 
                // that we no longer require loaded.

                notifier.unset_loading();
            }

            if can_free_memory_or_consume {
                debug!("Freeing decoded image from memory...");
                *image.decoded.lock().unwrap() = None;
            }

            *self.load_image_texture.lock().unwrap() = false;
            self.image_loading = false;
        }
    }
}