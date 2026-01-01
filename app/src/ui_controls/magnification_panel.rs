use eframe::egui::{self, Vec2};
use egui::Ui;

use crate::{viewport::Viewport};

pub struct MagnificationPanel {}

impl MagnificationPanel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(&mut self, ui: &Ui, viewport: &mut Viewport) {
        egui::Window::new("controls_window")
            .anchor(egui::Align2::RIGHT_CENTER, Vec2::new(-16.0, 0.0))
            .title_bar(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
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
                                viewport.zoom = (viewport.zoom + 0.2).clamp(1.0, 100.0);
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
                                viewport.zoom = (viewport.zoom - 0.2).clamp(1.0, 100.0);
                            }
                        });
                    });
            });
    }
}