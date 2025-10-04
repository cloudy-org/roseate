use cirrus_egui::v1::notifier::Notifier;
use eframe::egui::{self, Key, Vec2};
use egui_notify::ToastLevel;

use crate::{config::config::Config, zoom_pan::ZoomPan};

pub struct MagnificationPanel {
    pub show: bool,
    toggle_key: Key,
}

impl MagnificationPanel {
    pub fn new(config: &Config, notifier: &mut Notifier) -> Self {
        let toggle_key = match Key::from_name(&config.key_binds.ui_controls.toggle) {
            Some(key) => key,
            None => {
                notifier.toast(
                    "The key bind set for 'ui_controls.toggle' is invalid! Defaulting to `C`.", 
                    ToastLevel::Error,
                    |_| {}
                );

                Key::C
            },
        };

        Self {
            show: config.ui.magnification_panel.enabled_default,
            toggle_key,
        }
    }

    pub fn handle_input(&mut self, ctx: &egui::Context) {
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