use egui::{Align2, CursorIcon, Ui, Vec2, ViewportCommand};

pub struct FullscreenButtonPanel {}

impl FullscreenButtonPanel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(&mut self, ui: &Ui) {
        egui::Window::new("fullscreen_button_panel_window")
            .anchor(Align2::LEFT_BOTTOM, Vec2::new(16.0, -16.0))
            .title_bar(false)
            .resizable(false)
            .max_height(32.0)
            .show(ui.ctx(), |ui| {
                ui.horizontal_centered(|ui| {
                    let fullscreen_button = ui.add(
                        egui::Button::new("⛶")
                            .min_size(Vec2::new(40.0, 35.0))
                    ).on_hover_cursor(CursorIcon::PointingHand);

                    if fullscreen_button.clicked() {
                        let is_fullscreen = ui.ctx().input(
                            |i| i.viewport().fullscreen.unwrap_or_default()
                        );

                        ui.ctx().send_viewport_cmd(
                            ViewportCommand::Fullscreen(!is_fullscreen)
                        );
                    }
                });
            });
    }
}