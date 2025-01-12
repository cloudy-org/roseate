use std::{sync::{Arc, Mutex}, thread, time::{Duration, Instant}};

use log::{debug, info, warn};

use crate::{image::{backends::ImageProcessingBackend, image::Image, optimization::ImageOptimization}, notifier::NotifierAPI};

/// Struct that handles all the image loading logic in a thread safe 
/// manner to allow features such as background image loading / lazy loading.
pub struct ImageLoader {
    pub image_loaded: bool,

    image_loaded_arc: Arc<Mutex<bool>>,
    image_loading: bool,
}

impl ImageLoader {
    pub fn new() -> Self {
        Self {
            image_loaded: false,
            image_loaded_arc: Arc::new(Mutex::new(false)),
            image_loading: false,
        }
    }

    pub fn update(&mut self, ) {
        // I use an update function to keep the public fields update to date with their Arc<Mutex<T>> twins.

        if let Ok(value) = self.image_loaded_arc.try_lock() {
            self.image_loaded = value.clone(); // cloning here shouldn't too expensive
            self.image_loading = false;
        }
    }

    /// Handles loading the image in a background thread or on the main thread. 
    /// Set `lazy_load` to `true` if you want the image to be loaded in the background on a separate thread.
    /// 
    /// Setting `lazy_load` to `false` **will block the main thread** until the image is loaded.
    pub fn load_image(&mut self, image: &mut Image, lazy_load: bool, notifier: &mut NotifierAPI, use_experimental_backend: bool) {
        if self.image_loading {
            warn!("Not loading image as one is already being loaded!");
            return;
        }

        self.image_loading = true;

        notifier.set_loading(
            Some("Preparing to load image...".into())
        );

        let mut image = image.clone();

        notifier.set_loading(
            Some("Applying image optimizations...".into())
        );
        // optimizations = apply_image_optimizations(optimizations, &image.image_size);

        let mut optimizations = self.get_image_optimisations();

        // Our svg implementation is very experimental. Let's warn the user.
        if image.image_path.extension().unwrap_or_default() == "svg" {
            notifier.toasts.lock().unwrap()
                .toast_and_log(
                    "SVG files are experimental! \
                    Expect many bugs, inconstancies and performance issues.".into(),
                egui_notify::ToastLevel::Warning
                )
                .duration(Some(Duration::from_secs(8)));

            // SVGs cannot be loaded with optimizations at 
            // the moment or else image.load_image() will panic.
            optimizations.clear();
        }

        image.optimizations.extend(optimizations);

        let image_loaded_arc = self.image_loaded_arc.clone();
        let mut notifier_arc = notifier.clone();

        let mut loading_logic = move || {
            let backend = match use_experimental_backend {
                true => ImageProcessingBackend::Roseate,
                false => ImageProcessingBackend::ImageRS
            };

            notifier_arc.set_loading(Some("Loading image...".into()));
            let now = Instant::now();
            let result = image.load_image(
                &mut notifier_arc, 
                &backend
            );

            info!(
                "Image loaded in '{}' seconds using '{}' backend.", 
                now.elapsed().as_secs_f32(), backend
            );

            if let Err(error) = result {
                notifier_arc.toasts.lock().unwrap()
                    .toast_and_log(error.into(), egui_notify::ToastLevel::Error)
                    .duration(Some(Duration::from_secs(10)));
            }

            notifier_arc.unset_loading();
            *image_loaded_arc.lock().unwrap() = true;
        };

        if lazy_load {
            debug!("Lazy loading image (in a thread)...");
            thread::spawn(loading_logic);
        } else {
            debug!("Loading image in main thread...");
            loading_logic()
        }
    }

    // TODO: Make it apply optimizations following 
    // the user's config and other various factors.
    fn get_image_optimisations(&self) -> Vec<ImageOptimization> {
        let mut optimizations = Vec::new();

        // NOTE: wip, so just returning some basic optimizations for testing sake
        optimizations.push(ImageOptimization::MonitorDownsampling(130));

        optimizations
    }

}