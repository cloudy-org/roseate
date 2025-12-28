use egui::{Context, Id, LayerId, Popup, PopupAnchor, PopupCloseBehavior, Response, RichText, Ui};
use std::sync::{Arc, Mutex};

use roseate_core::decoded_image::DecodedImageInfo;

use crate::{Image, image_handler::{optimization::ImageOptimizations, resource::ImageResource}, windows::{ImageInfoWindow, WindowsManager}};

pub struct ContextMenu {
    info_window: Arc<Mutex<ImageInfoWindow>>,
    pointer_pos: Option<PopupAnchor>,

    show_menu: bool
}

impl ContextMenu {
    pub fn new(info_window: Arc<Mutex<ImageInfoWindow>>) -> Self {
        Self {
            info_window,
            pointer_pos: None,

            show_menu: false,
        }
    }

    fn reset(&mut self) {
        self.show_menu = false;
        self.pointer_pos = None;
    }

    pub fn handle_input(&mut self, ctx: &Context) {
        if ctx.input(|i| i.pointer.secondary_released()) {
            self.show_menu = true;
        }
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
    ) {
        if self.show_menu {
            let anchor = match self.pointer_pos {
                Some(a) => a,
                None => {
                    let pos2 = ui.ctx().pointer_latest_pos().unwrap();
                    self.pointer_pos = Some(PopupAnchor::Position(pos2));

                    PopupAnchor::Position(pos2)
                }
            };

            let id = Id::new("context_menu");
            Popup::new(id, ui.ctx().clone(), anchor, LayerId::new(egui::Order::Foreground, id))
                .show(|pop_ui| {
                    pop_ui.heading(RichText::new("Context menu"));
                    pop_ui.add_space(10.0);

                    let mut close = false;

                    if pop_ui.button("Image info").clicked() {
                        let mut info_window = self.info_window.lock().expect("Info window lock is poisoned");
                        info_window.set_to_show((true, false));

                        close = true;
                    }

                    if pop_ui.button("Image info with extra").clicked() {
                        let mut info_window = self.info_window.lock().expect("Info window lock is poisoned");
                        info_window.set_to_show((true, true));

                        close = true;
                    }

                    if close {
                        self.reset();
                    }
                });
        }
    }
}
