use cirrus_egui::v1::notifier::Notifier;
use egui::{Context, Key, Rect, Ui};
use egui_notify::ToastLevel;
use roseate_core::image_info::info::ImageInfo;

use crate::{image::Image, image_handler::{optimization::ImageOptimizations, resource::ImageResource}, utils::ctx_input_with_soft_binds};

mod info;
pub use info::ImageInfoWindow;

pub struct WindowsManager {
    info_window: ImageInfoWindow,

    pub show_info: bool,
    pub show_extra_info: bool,

    pub rect: Rect
}

impl WindowsManager {
    pub fn new() -> Self {
        let info_window = ImageInfoWindow::new();

        Self {
            info_window,

            show_info: false,
            show_extra_info: false,

            rect: Rect::NOTHING
        }
    }

    pub fn handle_input(
        &mut self,
        ctx: &Context,
        notifier: &mut Notifier,
        show_image_info_key: &String,
        show_extra_image_info_key: &String
    ) {
        // NOTE: in the future I'll probably turn this pattern into macros.
        let show_image_info = match ctx_input_with_soft_binds(
            ctx,
            show_image_info_key,
            |i, key| i.key_pressed(key),
            |i, modifiers| i.modifiers.contains(modifiers),
        ) {
            Ok(bool) => bool,
            Err(error) => {
                notifier.toast(
                    Box::new(error), ToastLevel::Error, |_| {}
                );

                ctx.input(|i| i.key_pressed(Key::I))
            },
        };

        let show_extra_image_info = match ctx_input_with_soft_binds(
            ctx,
            show_extra_image_info_key,
            |i, key| i.key_pressed(key),
            |i, modifiers| i.modifiers.contains(modifiers),
        ) {
            Ok(bool) => bool,
            Err(error) => {
                notifier.toast(
                    Box::new(error), ToastLevel::Error, |_| {}
                );

                ctx.input(|i| i.key_pressed(Key::I) && i.modifiers.ctrl)
            },
        };

        if show_image_info || show_extra_image_info {
            self.show_extra_info = show_extra_image_info;

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
