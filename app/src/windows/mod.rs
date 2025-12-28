use egui::{Context, Key, Ui};
use roseate_core::decoded_image::DecodedImageInfo;

use crate::{image::Image, image_handler::{optimization::ImageOptimizations, resource::ImageResource}};

use std::sync::{Arc, Mutex};

mod info;
pub use info::ImageInfoWindow;

pub struct WindowsManager {
    info_window: Arc<Mutex<ImageInfoWindow>>,

    show_info: bool,
    show_extra_info: bool,
}

impl WindowsManager {
    pub fn new() -> Self {
        let info_window = Arc::new(Mutex::new(ImageInfoWindow::new()));

        Self {
            info_window,

            show_info: false,
            show_extra_info: false,
        }
    }

    pub fn get_info_window(&self) -> Arc<Mutex<ImageInfoWindow>> {
        self.info_window.clone()
    }

    pub fn handle_input(&mut self, ctx: &Context) {
        if ctx.input(|i| i.key_pressed(Key::I)) {
            self.show_extra_info = ctx.input(|i| i.modifiers.ctrl);

            self.show_info = !self.show_info;
        }
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        image_resource: &ImageResource,
        image_optimizations: &ImageOptimizations,
        image: &Image,
        decoded_image_info: &DecodedImageInfo,
    ) {
        let mut info_window = self.info_window.lock().expect("Info window lock is poisoned");
        if let Some(tuple) = info_window.set_to_show {
            self.show_info = tuple.0;
            self.show_extra_info = tuple.1;

            info_window.set_to_show = None;
        }

        if self.show_info {
            info_window.show(
                ui,
                image_resource,
                image_optimizations,
                image,
                decoded_image_info,
                self.show_extra_info,
            );
        }
    }
}
