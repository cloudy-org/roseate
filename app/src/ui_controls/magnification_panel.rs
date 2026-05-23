use cirrus_egui::widgets::overlayer_banner::OverlayerBanner;
use eframe::egui::{self, Vec2};
use egui::{Align2, CursorIcon, Ui};

use crate::{viewport::Viewport};

pub struct MagnificationPanel {}

impl MagnificationPanel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(&mut self, ui: &Ui, viewport: &mut Viewport) {
        egui::Window::new("mag_panel_window")
            .anchor(Align2::RIGHT_CENTER, Vec2::new(-16.0, 0.0))
            .title_bar(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                let window_margin_spacing = ui.style().spacing.window_margin.bottomf();

                egui::Grid::new("mag_panel_grid")
                    .num_columns(2)
                    .spacing([0.0, window_margin_spacing])
                    .show(ui, |ui| {
                        let button_size = Vec2::new(20.0, 30.0);

                        ui.centered_and_justified(|ui| {
                            let zoom_in =
                                ui.add(
                                    egui::Button::new("+")
                                    .min_size(button_size)
                                ).on_hover_cursor(CursorIcon::PointingHand);

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
                                ).on_hover_cursor(CursorIcon::PointingHand);

                            if zoom_out.clicked() {
                                viewport.zoom = (viewport.zoom - 0.2).clamp(1.0, 100.0);
                            }
                        });
                    });
            });
    }
}