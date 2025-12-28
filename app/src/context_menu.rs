use egui::{Context, Id, LayerId, Popup, PopupAnchor, PopupCloseBehavior, Response, RichText, Ui};
use std::sync::{Arc, Mutex};

use roseate_core::decoded_image::DecodedImageInfo;

use crate::{Image, image_handler::{optimization::ImageOptimizations, resource::ImageResource}, windows::{ImageInfoWindow, WindowsManager}};

pub struct ContextMenu {
    info_window: Arc<Mutex<ImageInfoWindow>>,

    ignore_close: bool,
    pointer_pos: Option<PopupAnchor>,

    show_menu: bool
}

impl ContextMenu {
    pub fn new(info_window: Arc<Mutex<ImageInfoWindow>>) -> Self {
        Self {
            info_window,

            ignore_close: false,
            pointer_pos: None,

            show_menu: false,
        }
    }

    fn reset(&mut self) {
        self.show_menu = false;
        self.ignore_close = false;
        self.pointer_pos = None;
    }

    pub fn handle_input(&mut self, ctx: &Context) {
        if ctx.input(|i| i.pointer.secondary_released()) {
            self.show_menu = true;

            if self.pointer_pos.is_some() {
                self.ignore_close = true;
            }

            let pos2 = ctx.pointer_latest_pos().unwrap();
            self.pointer_pos = Some(PopupAnchor::Position(pos2));
        }
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
    ) {
        if self.show_menu {
            let anchor = self.pointer_pos.unwrap();

            let id = Id::new("context_menu");
            let response = Popup::new(id, ui.ctx().clone(), anchor, LayerId::new(egui::Order::Foreground, id))
                .close_behavior(PopupCloseBehavior::CloseOnClick)
                .show(|pop_ui| {
                    pop_ui.heading(RichText::new("Context menu"));
                    pop_ui.add_space(10.0);

                    if pop_ui.button("Image info").clicked() {
                        let mut info_window = self.info_window.lock().expect("Info window lock is poisoned");
                        info_window.set_to_show((true, false));
                    }

                    if pop_ui.button("Image info with extra").clicked() {
                        let mut info_window = self.info_window.lock().expect("Info window lock is poisoned");
                        info_window.set_to_show((true, true));
                    }
                });

            if let Some(response) = response {
                if response.response.should_close() && !self.ignore_close {
                    self.reset();
                }

                self.ignore_close = false;
            }
        }
    }
}
