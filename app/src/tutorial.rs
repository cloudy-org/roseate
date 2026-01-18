use cirrus_egui::v1::config_manager::ConfigManager;
use egui::{Align2, Area, Id, Order, Sense, TextWrapMode, Ui, UiKind, Vec2, include_image};

use crate::config::config::Config;

pub struct Tutorial {}

impl Tutorial {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(&mut self, ui: &mut Ui, config_manager: &mut ConfigManager<Config>) {
        // TODO: https://github.com/cloudy-org/roseate/issues/103 

        // if config_manager.config.misc.experimental.show_ui_modes_popup {
        //     let id = Id::new("ui_modes_popup");

        //     egui::Modal::new(id)
        //         .show(ui.ctx(), |ui| {
        //             ui.set_max_width(ui.ctx().content_rect().width().min(600.0) - 100.0);

        //             ui.horizontal_wrapped(|ui| {

        //                 ui.vertical(|ui| {
        //                     ui.add(
        //                         egui::Button::image(
        //                             egui::Image::new(include_image!("../assets/dev_preview_1.png"))
        //                                 .max_width(300.0),
        //                         )
        //                     );

        //                     ui.label("Standard Mode");
        //                 });

        //                 ui.vertical(|ui| {
        //                     ui.add(
        //                         egui::Button::image(
        //                             egui::Image::new(include_image!("../assets/dev_preview_1.png"))
        //                                 .max_width(300.0),
        //                         )
        //                     );

        //                     ui.label("Minimalist Mode");
        //                 });
        //             });

        //             ui.vertical_centered(|ui| {
        //                 if ui.button("Cancel").clicked() {
        //                     config_manager.config.misc.experimental.show_ui_modes_popup = false;

        //                     config_manager.save();
        //                 }
        //             });
        //         });

        //     ui.ctx().request_repaint();
        // }
    }
}
