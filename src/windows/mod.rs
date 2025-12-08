use egui::{Context, Key, Ui};

use crate::{image::image::Image, image_handler::ImageHandlerData, windows::info::ImageInfoWindow};

mod info;

pub struct WindowsManager {
    info_window: ImageInfoWindow,

    show_info: bool,
    show_extra_info: bool,
}

impl WindowsManager {
    pub fn new() -> Self {
        let info_window = ImageInfoWindow::new();

        Self {
            info_window,

            show_info: false,
            show_extra_info: false,
        }
    }

    pub fn handle_input(&mut self, ctx: &Context) {
        if ctx.input(|i| i.key_pressed(Key::I)) {
            self.show_extra_info = ctx.input(|i| i.modifiers.ctrl);

            self.show_info = !self.show_info;
        }
    }

    pub fn show(&mut self, ui: &mut Ui, image_handler_data: &ImageHandlerData, image: &Image) {
        if self.show_info {
            self.info_window.show(
                ui,
                image_handler_data,
                image,
                self.show_extra_info
            );
        }
    }
}