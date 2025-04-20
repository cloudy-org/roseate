use egui_notify::ToastLevel;
use cirrus_egui::v1::widgets::about::{authors_toml_to_about_authors, About, AboutApplicationInfo};
use eframe::egui::{self, Key, Response, Vec2};

use crate::{config::config::Config, files, notifier::NotifierAPI};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHORS: &str = include_str!("../../authors.toml");

pub struct AboutWindow<'a> {
    pub show: bool,
    toggle_key: Key,
    about_widget: About<'a>,
    pub response: Option<Response>,
}

impl<'a> AboutWindow<'a> {
    pub fn new(config: &Config, notifier: &mut NotifierAPI) -> Self {
        let config_key = match Key::from_name(&config.key_binds.about_box.toggle) {
            Some(key) => key,
            None => {
                notifier.toasts.lock().unwrap().toast_and_log(
                    "The key bind set for 'about_box.toggle' is invalid! Defaulting to `A`.".into(), 
                    ToastLevel::Error
                );

                Key::A
            },
        };

        let about_app_info = AboutApplicationInfo {
            name: "Roseate".to_string(),
            description: "Fast and minimal GPU accelerated image viewer that's cross platform.".to_string(),
            license: include_str!("../../LICENSE").to_string(),
            version: VERSION.to_string(),
            authors: authors_toml_to_about_authors(&AUTHORS.to_string()),
            webpage: "https://github.com/cloudy-org/roseate".to_string(),
            git_repo: "https://github.com/cloudy-org/roseate".to_string(),
            copyright: "Copyright (C) 2024 - 2025 Goldy".to_string()
        };

        let about_widget = About::new(
            files::get_platform_rose_image(), about_app_info
        );

        Self {
            show: false,
            about_widget,
            toggle_key: config_key,
            response: None
        }
    }

    // TODO: I see we repeat this a lot, we should find a way to modularize self.config_key 
    // and this function in a way we can easily add it to the struct with very very minimal lines.
    //
    // I'm trying to investigate if there's any rust features that will 
    // allow me to do this with one line like `impl Default for Struct`.
    pub fn handle_input(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.key_pressed(self.toggle_key)) {
            if self.show == true {
                self.show = false;
            } else {
                self.show = true;
            }
        }
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        if self.show {
            let default_window_size = Vec2::new(340.0, 350.0);

            let response = egui::Window::new(
                egui::WidgetText::RichText(
                    egui::RichText::new("ℹ About").size(15.0)
                )
            )
                .default_size(default_window_size)
                .min_width(270.0)
                // NOTE: doesn't actually center probably, yes I know, 
                // has been fixed but not released yet: https://github.com/emilk/egui/issues/5314
                .default_pos(ctx.screen_rect().center() - default_window_size / 2.0)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        self.about_widget.show(ctx, ui);
                    });
                });

            self.response = Some(response.unwrap().response);
        }

        self.about_widget.update(ctx);
    }
}