use cirrus_authors::Authors;
use cirrus_egui::{widgets::about::{About, AboutApplicationInfo}};
use eframe::egui::{self, Response, Vec2};
use egui::Ui;

use crate::files;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const REPO_URL: &str = env!("CARGO_PKG_REPOSITORY");
const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

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
                                name: String::from("Roseate"),
                                description: DESCRIPTION.to_string(),
                                license: include_str!("../../LICENSE").to_string(),
                                version: VERSION.to_string(),
                                authors: authors,
                                webpage: REPO_URL.to_string(),
                                git_repo: REPO_URL.to_string(),
                                copyright: String::from("Copyright (C) 2024 - 2026 Goldy")
                            },
                            show_license
                        );
                });
            });

        response.unwrap().response
    }
}