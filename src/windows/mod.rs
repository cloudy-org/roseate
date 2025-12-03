use egui::{Context, Key, Ui};

use crate::{image::image::Image, windows::{info::InfoWindow}};

mod info;

pub struct WindowsManager {
    info_window: InfoWindow,

    show_info: bool,
}

impl WindowsManager {
    pub fn new() -> Self {
        let info_window = InfoWindow::new();

        Self {
            info_window,

            show_info: false
        }
    }

    pub fn handle_input(&mut self, ctx: &Context) {
        if ctx.input(|i| i.key_pressed(Key::I)) {
            self.show_info = !self.show_info;
        }
    }

    pub fn show(&mut self, ui: &mut Ui, image: &Image) {
        if self.show_info { self.info_window.show(ui, image); }
    }
}