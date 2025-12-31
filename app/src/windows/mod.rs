use egui::{Context, Key, Rect, Ui};
use roseate_core::image_info::info::ImageInfo;

use crate::{image::Image, image_handler::{optimization::ImageOptimizations, resource::ImageResource}};

mod info;
pub use info::ImageInfoWindow;

pub struct WindowsManager {
    info_window: ImageInfoWindow,

    pub show_info: bool,
    pub show_extra_info: bool,

    pub rect: Rect
}

impl WindowsManager {
    pub fn new(show_location: bool) -> Self {
        let info_window = ImageInfoWindow::new(show_location);

        Self {
            info_window,

            show_info: false,
            show_extra_info: false,

            rect: Rect::NOTHING
        }
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
        image_info: &ImageInfo,
    ) {
        let mut new_rect: Rect = Rect::NOTHING;

        if self.show_info {
            let response = self.info_window.show(
                ui,
                image_resource,
                image_optimizations,
                image,
                image_info,
                self.show_extra_info,
            );

            new_rect = new_rect.union(response.rect);
        }

        self.rect = new_rect;
    }
}
