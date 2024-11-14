use std::{sync::{Arc, Mutex}, thread, time::Duration};

use egui_notify::Toast;
use log::{debug, warn};

use crate::{image::{apply_image_optimizations, Image}, toasts::ToastsManager};

#[derive(Default, Clone)]
pub struct Loading {
    pub message: Option<String>
}

// just wanted to play around with these, see how I use them below
macro_rules! loading_msg {
    ($message_string: expr, $image_loading_arc: ident) => {
        {
            *$image_loading_arc.lock().unwrap() = Some(
                Loading {
                    message: Some($message_string.into())
                }
            );
        }
    }
}

/// Struct that handles all the image loading logic in a thread safe 
/// manner to allow features such as background image loading / lazy loading.
pub struct ImageLoader {
    toasts_queue_arc: Arc<Mutex<Vec<Toast>>>,

    pub image_loaded: bool,
    image_loaded_arc: Arc<Mutex<bool>>,
    pub image_loading: Option<Loading>,
    image_loading_arc: Arc<Mutex<Option<Loading>>>,
}

impl ImageLoader {
    pub fn new() -> Self {
        Self {
            toasts_queue_arc: Arc::new(Mutex::new(Vec::new())),
            image_loaded: false,
            image_loaded_arc: Arc::new(Mutex::new(false)),
            image_loading: None,
            image_loading_arc: Arc::new(Mutex::new(None))
        }
    }

    pub fn update(&mut self, toasts: &mut ToastsManager) {
        // I use an update function to keep the public fields update to date with their Arc<Mutex<T>> twins.
        //
        // I also use this to append the queued toast messages 
        // from threads as we cannot take ownership of "&mut Toasts" sadly.

        if let Ok(value) = self.image_loading_arc.try_lock() {
            self.image_loading = value.clone(); // TODO: find a way to reference instead of clone to save memory here.
        }

        if let Ok(value) = self.image_loaded_arc.try_lock() {
            self.image_loaded = value.clone(); // TODO: find a way to reference instead of clone to save memory here.
        }

        if let Ok(mut queue) = self.toasts_queue_arc.try_lock() {
            for toast in queue.drain(..) {
                toasts.toasts.add(toast);
            }
        }
    }

    /// Handles loading the image in a background thread or on the main thread. 
    /// Set `lazy_load` to `true` if you want the image to be loaded in the background on a separate thread.
    /// 
    /// Setting `lazy_load` to `false` **will block the main thread** until the image is loaded.
    pub fn load_image(&mut self, image: &mut Image, lazy_load: bool) {
        if self.image_loading_arc.lock().unwrap().is_some() {
            warn!("Not loading image as one is already being loaded!");
            return;
        }

        *self.image_loading_arc.lock().unwrap() = Some(
            Loading {
                message: Some("Preparing to load image...".into())
            }
        );

        let mut image = image.clone();

        // Our svg implementation is very experimental. Let's warn the user.
        if image.image_path.extension().unwrap_or_default() == "svg" {
            let msg = "SVG files are experimental! \
                Expect many bugs, inconstancies and performance issues.";

            let mut toast = Toast::warning(msg);
            toast.duration(Some(Duration::from_secs(8)));

            self.toasts_queue_arc.lock().unwrap().push(toast);

            warn!("{}", msg);
        }

        let toasts_queue_arc = self.toasts_queue_arc.clone();
        let image_loaded_arc = self.image_loaded_arc.clone();
        let image_loading_arc = self.image_loading_arc.clone();

        let mut loading_logic = move || {
            let mut optimizations = Vec::new();

            loading_msg!("Applying image optimizations...", image_loading_arc);
            optimizations = apply_image_optimizations(optimizations, &image.image_size);

            loading_msg!("Loading image...", image_loading_arc);
            let result = image.load_image(&optimizations);

            if let Err(error) = result {
                let mut toast = Toast::error(
                    textwrap::wrap(&error.message(), 100).join("\n")
                );
                toast.duration(Some(Duration::from_secs(10)));

                toasts_queue_arc.lock().unwrap().push(toast);

                log::error!("{}", error.message());
            }

            let mut image_loaded = image_loaded_arc.lock().unwrap();
            let mut image_loading = image_loading_arc.lock().unwrap();

            *image_loaded = true;
            *image_loading = None;
        };

        if lazy_load {
            debug!("Lazy loading image (in a thread)...");
            thread::spawn(loading_logic);
        } else {
            debug!("Loading image in main thread...");
            loading_logic()
        }
    }
}