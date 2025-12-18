use std::{collections::HashSet, sync::{Arc, Mutex}, thread, time::{Duration, Instant}};

use cirrus_egui::v1::{notifier::Notifier, scheduler::Scheduler};
use egui::Context;
use log::{debug, info, warn};
use roseate_core::{decoded_image::ImageSize, format::ImageFormat, modifications::{ImageModification, ImageModifications}};

use crate::{image::{Image, backend::DecodingBackend}, image_handler::{optimization::ImageOptimizations, resource::ImageResource}, monitor_size::MonitorSize};

pub struct ImageHandler {
    pub image: Option<Image>,
    pub resource: Option<ImageResource>,

    pub image_loading: bool,

    pub(super) image_optimizations: ImageOptimizations,
    pub(super) dynamic_sample_schedule: Option<Scheduler>,
    pub(super) last_zoom_factor: f32,
    pub(super) dynamic_sampling_new_resolution: ImageSize,
    pub(super) dynamic_sampling_old_resolution: ImageSize,
    pub(super) accumulated_zoom_factor_change: f32,
    pub(super) monitor_downsampling_required: bool,
    pub(super) load_image_texture: Arc<Mutex<bool>>,
}

impl ImageHandler {
    pub fn new(image: Option<Image>, image_optimizations: ImageOptimizations) -> Self {
        Self {
            image: image,
            image_optimizations,

            resource: None,
            image_loading: false,

            dynamic_sample_schedule: None,
            last_zoom_factor: 1.0,
            dynamic_sampling_new_resolution: (0, 0),
            dynamic_sampling_old_resolution: (0, 0),
            accumulated_zoom_factor_change: 0.0,
            monitor_downsampling_required: false,

            load_image_texture: Arc::new(Mutex::new(false))
        }
    }

    pub fn update(
        &mut self,
        ctx: &Context,
        zoom_factor: &f32,
        is_panning: bool,
        monitor_size: &MonitorSize,
        backend: DecodingBackend,
        notifier: &mut Notifier,
    ) {
        self.load_resource_update(ctx);
        self.dynamic_sampling_update(zoom_factor, monitor_size);

        if self.image.is_some() {

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
        lazy_load: bool,
        backend: DecodingBackend,
        monitor_size: &MonitorSize,
        notifier: &mut Notifier
    ) {
        if self.image_loading { // would we ever even hit this?
            warn!("Not loading image as one is already being loaded!");
            return;
        }

        if let Some(image) = self.image.clone() {
            let image_modifications = self.get_image_modifications(
                &image.size,
                monitor_size,
            );

            self.image_loading = true;

            notifier.set_loading(
                Some("Gathering necessary image modifications...")
            );

            let image_modifications_debug = format!("{:?}", image_modifications);

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
            let load_image_texture_clone = self.load_image_texture.clone();

            let image_loaded = self.resource.is_some();

            let loading_logic = move || {
                let now = Instant::now();

                let result = match image_loaded {
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
                        *load_image_texture_clone.lock().unwrap() = true;

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


    }

    /// Method that handles choosing which type of modifications 
    /// should be done to the image at this time. It decides that on a number of various factors, 
    /// like image optimizations applied by the user, monitor size, zoom factor and etc.
    fn get_image_modifications(&mut self, image_size: &ImageSize, monitor_size: &MonitorSize) -> ImageModifications {
        let mut image_modifications = HashSet::new();

        if let Some(monitor_downsampling) = &self.image_optimizations.monitor_downsampling {
            let (width, height) = monitor_downsampling.get_size_relative_to_monitor(&monitor_size);

            // If the image is a lot bigger than the user's 
            // monitor then apply monitor downsample, if not we shouldn't.
            if image_size.0 > width && image_size.1 > height {
                self.monitor_downsampling_required = true;

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

                image_modifications.replace(ImageModification::Resize(width, height));
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