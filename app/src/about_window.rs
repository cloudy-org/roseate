use cirrus_authors::Authors;
use cirrus_egui::{widgets::about::{About, AboutApplicationInfo}};
use eframe::egui::{self, Response, Vec2};
use egui::Ui;

use crate::files;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct AboutWindow {}

impl AboutWindow {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(&mut self, ui: &Ui, authors: &Authors, show_license: &mut bool) -> Response {
        let default_window_size = Vec2::new(340.0, 350.0);

        let ctx = ui.ctx();

        let response = egui::Window::new(
            egui::WidgetText::RichText(
                egui::RichText::new("ℹ About").size(15.0).into()
            )
        )
            .default_size(default_window_size)
            .min_width(270.0)
            .default_pos(ctx.content_rect().center() - default_window_size / 2.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    About::new()
                        .show(
                            ui,
                            files::get_rose_image(),
                            AboutApplicationInfo {
                                name: "Roseate".to_string(),
                                description: "Fancy yet simple image viewer — highly configurable, \
                                    cross-platform, GPU-accelerated and fast as fu#k.".to_string(),
                                license: include_str!("../../LICENSE").to_string(),
                                version: VERSION.to_string(),
                                authors: authors,
                                webpage: "https://github.com/cloudy-org/roseate".to_string(),
                                git_repo: "https://github.com/cloudy-org/roseate".to_string(),
                                copyright: "Copyright (C) 2024 - 2026 Goldy".to_string()
                            },
                            show_license
                        );
                });
            });

        response.unwrap().response
    }
}