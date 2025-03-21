use std::{path::Path, sync::{Arc, Mutex}, thread, time::{Duration, Instant}};

use log::{debug, info, warn};
use rfd::FileDialog;

use crate::{error::{Error, Result}, image::{backends::ImageProcessingBackend, image::Image, optimization::{EventImageOptimizations, ImageOptimizations}}, monitor_size::MonitorSize, notifier::NotifierAPI, zoom_pan::ZoomPan};

/// Struct that handles all the image loading logic in a thread safe 
/// manner to allow features such as background image loading / lazy loading.
/// 
/// ImageHandler struct is a ui facing, Image struct is low-level stuff.
pub struct ImageHandler {
    pub image: Option<Image>,
    pub image_loaded: bool,

    image_loaded_arc: Arc<Mutex<bool>>,
    image_loading: bool,

    last_zoom_factor: f32,
    accumulated_zoom_factor: f32
}

impl ImageHandler {
    pub fn new() -> Self {
        Self {
            image: None,
            image_loaded: false,
            image_loaded_arc: Arc::new(Mutex::new(false)),
            image_loading: false,
            last_zoom_factor: 1.0,
            accumulated_zoom_factor: 0.0,
        }
    }

    pub fn init_image(&mut self, image_path: &Path, monitor_size: &MonitorSize) -> Result<()> {
        let mut image = Image::from_path(image_path)?;

        image.optimizations.extend(self.get_user_image_optimisations(&image, monitor_size));

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

                self.init_image(&path, monitor_size)?;

                Ok(())
            },
            None => Err(Error::NoFileSelected(None))
        }
    }

    pub fn update(&mut self, zoom_pan: &ZoomPan) {
        // I use an update function to keep the public 
        // fields update to date with their Arc<Mutex<T>> twins
        // and also now to perform dynamic downsampling.

        if let Ok(value) = self.image_loaded_arc.try_lock() {
            self.image_loaded = value.clone(); // cloning here shouldn't be too expensive
            self.image_loading = false; // set that bitch back to false yeah
        }

        // TODO: perform dynamic downsampling here! (25/01/2025)
        self.dynamic_sampling_update(zoom_pan);
    }

    pub fn dynamic_sampling_update(&mut self, zoom_pan: &ZoomPan) {
        if let Some(image) = &self.image {
            // the zoom factor change since the last dynamic upsample / downsample.
            let zoom_factor_change = zoom_pan.zoom_factor - self.last_zoom_factor;

            let is_enabled = image.optimizations.contains(
                &ImageOptimizations::EventBased(EventImageOptimizations::DynamicUpsampling)
            );

            if !is_enabled || !(zoom_factor_change <= -0.4) && !(zoom_factor_change >= 0.4) {
                return;
            }

            if zoom_pan.zoom_factor <= 1.0 {
                return;
            }

            // TODO: (20/03/2025) use accumulated_zoom_factor to determin when to do a dynamic sample.
            // we only want the image to sample when the user zooms into the image far enough for the 
            // quality to drop. (e.g: micro zooms should not trigger samples unless many of them have 
            // been done overall accumulating to a "far enough" zoom)

            println!("uwu {}", zoom_pan.zoom_factor);

            // TODO: also don't upsample if image is already at it's max resolution.
            if zoom_factor_change >= 0.4 {
                // TODO: reload image with new dimensions

                // TODO: fix the weird bug where this doesn't always trigger when zoom factor is reset by roseate.

                println!("owo + {}", zoom_factor_change);
            }

            if zoom_factor_change <= -0.4 {
                println!("owo - {}", zoom_factor_change);
            }

            self.last_zoom_factor = zoom_pan.zoom_factor;
        }
    }

    /// Handles loading the image in a background thread or on the main thread. 
    /// Set `lazy_load` to `true` if you want the image to be loaded in the background on a separate thread.
    /// 
    /// Setting `lazy_load` to `false` **will block the main thread** until the image is loaded.
    pub fn load_image(&mut self, lazy_load: bool, notifier: &mut NotifierAPI, monitor_size: &MonitorSize, use_experimental_backend: bool) {
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
            Some("Applying image optimizations...".into())
        );

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
            image.optimizations.clear();
        }

        let image_loaded_arc = self.image_loaded_arc.clone();
        let mut notifier_arc = notifier.clone();
        let monitor_size_arc = monitor_size.clone();

        let mut loading_logic = move || {
            let backend = match use_experimental_backend {
                true => ImageProcessingBackend::Roseate,
                false => ImageProcessingBackend::ImageRS
            };

            notifier_arc.set_loading(Some("Loading image...".into()));
            let now = Instant::now();
            let result = image.load_image(
                &mut notifier_arc,
                &monitor_size_arc,
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

    // TODO: Make it apply optimizations following the user's config.
    fn get_user_image_optimisations(&self, image: &Image, monitor_size: &MonitorSize) -> Vec<ImageOptimizations> {
        use crate::image::optimization::EventImageOptimizations::*;
        use crate::image::optimization::InitialImageOptimizations::*;

        let mut optimizations = Vec::new();

        let (monitor_width, monitor_height) = monitor_size.get();

        // If the image is a lot bigger than the user's 
        // monitor then apply monitor downsample, if not we shouldn't.
        if image.image_size.width as u32 > monitor_width as u32 && image.image_size.height as u32 > monitor_height as u32 {
            optimizations.push(ImageOptimizations::Initial(MonitorDownsampling(130)));
        }

        // NOTE: wip, so just returning some random optimizations for testing sake
        optimizations.push(ImageOptimizations::EventBased(DynamicUpsampling));

        optimizations
    }

}