use cirrus_egui::v1::notifier::Notifier;
use egui::{Context, InputState, Key, Rect, Ui};
use egui_notify::ToastLevel;
use roseate_core::image_info::info::ImageInfo;

use crate::{image::Image, image_handler::{optimization::ImageOptimizations, resource::ImageResource}, utils::{get_input_reader_from_soft_binds}};

mod info;
pub use info::ImageInfoWindow;

type InputReader = Box<dyn FnMut(&InputState) -> bool>;

pub struct WindowsManager {
    info_window: ImageInfoWindow,

    show_info_reader: Option<InputReader>,
    show_extra_info_reader: Option<InputReader>,

    pub show_info: bool,
    pub show_extra_info: bool,

    pub rect: Rect
}

impl WindowsManager {
    pub fn new() -> Self {
        let info_window = ImageInfoWindow::new();

        Self {
            info_window,

            show_info_reader: None,
            show_extra_info_reader: None,

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
        // TODO: put this into some nice function without loosing too much control somehow.
        let show_info_reader = self.show_info_reader.get_or_insert_with(|| {
            match get_input_reader_from_soft_binds(
                show_image_info_key,
                |i, key| i.key_pressed(key)
            ) {
                Ok(reader) => Box::new(reader),
                Err(error) => {
                    notifier.toast(
                        Box::new(error), ToastLevel::Error, |_| {}
                    );

                    Box::new(|i| i.key_pressed(Key::I))
                },
            }
        });

        let show_extra_info_reader = self.show_extra_info_reader.get_or_insert_with(|| {
            match get_input_reader_from_soft_binds(
                show_extra_image_info_key,
                |i, key| i.key_pressed(key)
            ) {
                Ok(reader) => Box::new(reader),
                Err(error) => {
                    notifier.toast(
                        Box::new(error), ToastLevel::Error, |_| {}
                    );

                    Box::new(|i| i.key_pressed(Key::I) && i.modifiers.ctrl)
                },
            }
        });

        let show_extra_info = ctx.input(show_extra_info_reader);

        if ctx.input(show_info_reader) || show_extra_info {
            self.show_extra_info = show_extra_info;

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
