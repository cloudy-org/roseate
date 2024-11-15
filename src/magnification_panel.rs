use eframe::egui::{self, Key, Vec2};
use egui_notify::ToastLevel;

use crate::{config::config::Config, toasts::ToastsManager, zoom_pan::ZoomPan};

pub struct MagnificationPanel {
    pub show: bool,
    toggle_key: Key,
}

impl MagnificationPanel {
    // TODO: When this branch is merged into main 
    // remove "image" from the initialization of this struct.
    pub fn new(config: &Config, toasts: &mut ToastsManager) -> Self {
        let toggle_key = match Key::from_name(&config.keybinds.ui_controls.toggle) {
            Some(key) => key,
            None => {
                toasts.toast_and_log(
                    "The key bind set for 'ui_controls.toggle' is invalid! Defaulting to `C`.".into(), 
                    ToastLevel::Error
                );

                Key::C
            },
        };

        Self {
            show: false,
            toggle_key,
        }
    }

    pub fn handle_input(&mut self, ctx: &egui::Context) {
        // NOTE: For now let's hide the magnification panel behind a keybind.
        // TODO: When the toml config is ready (https://github.com/cloudy-org/roseate/issues/20) 
        // we can add a settings to have it shown by default or not.

        if ctx.input(|i| i.key_pressed(self.toggle_key)) {
            if self.show == true {
                self.show = false;
            } else {
                self.show = true;
            }
        }
    }

    pub fn update(&mut self, ctx: &egui::Context, zoom_pan: &mut ZoomPan) {
        if !self.show {
            return;
        }

        egui::Window::new("controls_window")
            .anchor(egui::Align2::RIGHT_CENTER, Vec2::new(-16.0, 0.0))
            .title_bar(false)
            .resizable(false)
            .show(ctx, |ui| {
                egui::Grid::new("controls_grid")
                    .spacing([10.0, 10.0])
                    .num_columns(2)
                    .show(ui, |ui| {
                        let button_size = Vec2::new(20.0, 30.0);

                        ui.centered_and_justified(|ui| {
                            let zoom_in =
                                ui.add(
                                    egui::Button::new("+")
                                    .min_size(button_size)
                                );

                            if zoom_in.clicked() {
                                zoom_pan.zoom_factor = (zoom_pan.zoom_factor + 0.2).clamp(1.0, 100.0);
                            }
                        });
                        ui.end_row();

                        ui.centered_and_justified(|ui| {
                            let zoom_out =
                                ui.add(
                                    egui::Button::new("-")
                                    .min_size(button_size)
                                );

                            if zoom_out.clicked() {
                                zoom_pan.zoom_factor = (zoom_pan.zoom_factor - 0.2).clamp(1.0, 100.0);
                            }
                        });
                    });
            });
    }
}