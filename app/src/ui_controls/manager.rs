use std::time::Duration;

use cirrus_egui::{notifier::{Notifier, banner::{BannerPlacement, BannerText}}};
use cirrus_soft_binds::egui::{BoxedEguiInputReaderFunc, parse_and_get_egui_input_reader_from_string};
use eframe::egui::{Context, Key, Ui};
use egui_notify::ToastLevel;

use crate::{ui_controls::{fullscreen::FullscreenButton, magnification_panel::MagnificationPanel, settings::SettingsButton}, viewport::Viewport};

pub struct UIControlsManager {
    magnification_panel: MagnificationPanel,
    fullscreen_button: FullscreenButton,
    settings_button: SettingsButton,

    show_controls_reader: Option<BoxedEguiInputReaderFunc>,

    show_controls: Option<bool>,
}

impl UIControlsManager {
    pub fn new() -> Self {
        let magnification_panel = MagnificationPanel::new();
        let fullscreen_button = FullscreenButton::new();
        let settings_button = SettingsButton::new();

        Self {
            magnification_panel,
            fullscreen_button,
            settings_button,

            show_controls_reader: None,

            show_controls: None
        }
    }

    pub fn handle_input(
        &mut self,
        ctx: &Context,
        notifier: &mut Notifier,
        show_controls_key: &String,
        show_by_default: bool
    ) {
        let show_controls = self.show_controls.get_or_insert(show_by_default);

        let show_controls_reader = self.show_controls_reader.get_or_insert_with(
            || {
                match parse_and_get_egui_input_reader_from_string(
                    show_controls_key,
                    |i, key| i.key_pressed(key)
                ) {
                    Ok(reader) => Box::new(reader),
                    Err(error) => {
                        notifier.toast(
                            error.to_string(),
                            ToastLevel::Error,
                            |_| {}
                        );

                        Box::new(|i| i.key_pressed(Key::C))
                    },
                }
            }
        );

        if ctx.input(show_controls_reader) {
            *show_controls ^= true;

            notifier.show_banner(
                BannerText::new(
                    format!(
                        "{} UI controls ({show_controls_key})",
                        match show_controls {
                            true => "Show",
                            false => "Hide",
                        }
                    ),
                    None
                ),
                BannerPlacement::BOTTOM,
                Duration::from_secs(2)
            );
        }
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        viewport: &mut Viewport,

        show_magnification_panel: bool,
        show_fullscreen_button: bool,
        show_settings_button: bool,

        show_settings: &mut bool,
    ) {
        if self.show_controls.unwrap_or(false) {
            if show_magnification_panel {
                self.magnification_panel.show(ui, viewport);
            }

            if show_fullscreen_button {
                self.fullscreen_button.show(ui);
            }

            if show_settings_button {
                self.settings_button.show(ui, show_settings);
            }
        }
    }
}