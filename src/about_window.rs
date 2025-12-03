use cirrus_egui::v1::{widgets::about::{authors_toml_to_about_authors, About, AboutApplicationInfo}};
use eframe::egui::{self, Response, Vec2};
use egui::Ui;

use crate::files;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHORS: &str = include_str!("../authors.toml");

pub struct AboutWindow<'a> {
    about_widget: About<'a>,
}

impl<'a> AboutWindow<'a> {
    pub fn new() -> Self {
        // let config_key = match Key::from_name(&config.key_binds.about_box.toggle) {
        //     Some(key) => key,
        //     None => {
        //         notifier.toast(
        //             "The key bind set for 'about_box.toggle' is invalid! Defaulting to `A`.", 
        //             ToastLevel::Error,
        //             |_| {}
        //         );

        //         Key::A
        //     },
        // };

        let about_app_info = AboutApplicationInfo {
            name: "Roseate".to_string(),
            description: "Fast and minimal GPU accelerated image viewer that's cross platform.".to_string(),
            license: include_str!("../LICENSE").to_string(),
            version: VERSION.to_string(),
            authors: authors_toml_to_about_authors(&AUTHORS.to_string()),
            webpage: "https://github.com/cloudy-org/roseate".to_string(),
            git_repo: "https://github.com/cloudy-org/roseate".to_string(),
            copyright: "Copyright (C) 2024 - 2025 Goldy".to_string()
        };

        let about_widget = About::new(
            files::get_rose_image(), about_app_info
        );

        Self {
            about_widget,
        }
    }

    pub fn show(&mut self, ui: &Ui) -> Response {
        let default_window_size = Vec2::new(340.0, 350.0);

        let ctx = ui.ctx();

        let response = egui::Window::new(
            egui::WidgetText::RichText(
                egui::RichText::new("ℹ About").size(15.0).into()
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

        self.about_widget.update(ctx);

        response.unwrap().response
    }
}