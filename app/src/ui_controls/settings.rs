use egui::{Align2, Ui, Vec2};

use cirrus_egui::widgets::settings::button::SettingsButton as CirrusSettingsButton;

pub struct SettingsButton {}

impl SettingsButton {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(&mut self, ui: &Ui, show_settings: &mut bool) {
        egui::Window::new("settings_button_window")
            .anchor(Align2::RIGHT_TOP, Vec2::new(-16.0, 16.0))
            .title_bar(false)
            .resizable(false)
            .max_height(32.0)
            .show(ui.ctx(), |ui| {
                ui.horizontal_centered(|ui| {
                    CirrusSettingsButton::new()
                        .show(ui, show_settings);
                });
            });
    }
}