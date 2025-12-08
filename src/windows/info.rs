use std::alloc;

use cap::Cap;
use chrono::{DateTime, Local};
use egui::{Pos2, RichText, Ui, WidgetText};
use eframe::egui::{self, Response};

use crate::{image::image::Image, image_handler::ImageHandlerData};

#[global_allocator]
static ALLOCATOR: Cap<alloc::System> = Cap::new(alloc::System, usize::max_value());

pub struct ImageInfoWindow {
    data: Option<ImageInfoData>
}

impl ImageInfoWindow {
    pub fn new() -> Self {
        Self {
            data: None
        }
    }

    pub fn show<'a>(&mut self, ui: &Ui, image_handler_data: &ImageHandlerData, image: &Image, show_extra: bool) -> Response {
        let image_info_data = self.data.get_or_insert_with(
            || ImageInfoData::new(image_handler_data, image)
        );

        let window = egui::Window::new(
            WidgetText::RichText(
                match show_extra {
                    false => RichText::new("ℹ Image Info"),
                    true => RichText::new("ℹ Image Info (Extra)"),
                }.size(15.0).into()
            )
        );

        window.default_pos(Pos2::new(200.0, 200.0))
            .resizable(false)
            .fade_in(false)
            .fade_out(false)
            .show(ui.ctx(), |ui| {
                let app_memory_allocated = ALLOCATOR.allocated();

                egui::Frame::group(&ui.ctx().style())
                    .show(ui, |ui| {
                        egui::Grid::new("info_window_grid")
                            .num_columns(2)
                            .show(ui, |ui| {
                                egui::Grid::new("image_info_grid")
                                    .striped(true)
                                    .max_col_width(150.0)
                                    .show(ui, |ui| {
                                        ui.label("Name:");
                                        ui.label(&image_info_data.file_name);
                                        ui.end_row();

                                        ui.label("Dimensions:");
                                        ui.label(
                                            format!(
                                                "{}x{}", image.image_size.0, image.image_size.1
                                            )
                                        );
                                        ui.end_row();

                                        ui.label("Format:");
                                        ui.label(format!("{}", image.image_format));
                                        ui.end_row();

                                        ui.label("File Created:");
                                        ui.label(
                                            match &image_info_data.file_created_time {
                                                Some(time_string) => RichText::new(time_string),
                                                None => RichText::new("Unknown").weak(),
                                            }
                                        );
                                        ui.end_row();

                                        ui.label("File Modified:");
                                        ui.label(
                                            match &image_info_data.file_modified_time {
                                                Some(time_string) => RichText::new(time_string),
                                                None => RichText::new("Unknown").weak(),
                                            }
                                        );
                                        ui.end_row();

                                        ui.label("File size:");
                                        ui.label(
                                            match image_info_data.file_size {
                                                Some(size) => RichText::new(re_format::format_bytes(size)),
                                                None => RichText::new("Unknown").weak(),
                                            }
                                        );
                                        ui.end_row();
                                    });

                                if show_extra {
                                    egui::Grid::new("misc_info_grid")
                                        .striped(true)
                                        .max_col_width(150.0)
                                        .show(ui, |ui| {
                                            let mem_alloc_hint = "How much memory the entire app is currently allocating.";

                                            ui.label("Mem Alloc:").on_hover_text(mem_alloc_hint);
                                            ui.label(
                                                RichText::new(re_format::format_bytes(app_memory_allocated as f64))
                                                .code()
                                            ).on_hover_text(mem_alloc_hint);
                                            ui.end_row();
                                        });
                                }

                                ui.end_row();
                            });
                    }
                );
            }).unwrap().response
    }
}

struct ImageInfoData {
    pub file_name: String,
    pub file_size: Option<f64>,
    pub file_created_time: Option<String>,
    pub file_modified_time: Option<String>,
}

impl ImageInfoData {
    pub fn new(image_handler_data: &ImageHandlerData, image: &Image) -> Self {
        let path = &image.image_path;

        let file_name = path.file_name().unwrap().to_string_lossy().to_string();

        let file_metadata = match path.metadata() {
            Ok(metadata) => Some(metadata),
            Err(error) => {
                log::error!(
                    "Failed to retrive image file metadata from file system! Error: {}",
                    error
                );

                None
            },
        };

        let mut file_size = None;
        let mut file_created_time = None;
        let mut file_modified_time = None;

        if let Some(metadata) = file_metadata {
            file_created_time = match metadata.created() {
                Ok(time) => {
                    let datetime: DateTime<Local> = time.into();
                    Some(datetime.format("%d/%m/%Y (%H:%M)").to_string())
                },
                Err(error) => {
                    log::warn!("Failed to retrieve image file created date! Error: {}", error);

                    None
                },
            };

            file_modified_time = match metadata.modified() {
                Ok(time) => {
                    let datetime: DateTime<Local> = time.into();
                    Some(datetime.format("%d/%m/%Y (%H:%M)").to_string())
                },
                Err(error) => {
                    log::warn!("Failed to retrieve image file modified date! Error: {}", error);

                    None
                },
            };

            file_size = Some(metadata.len() as f64);
        }

        Self {
            file_name,
            file_size,
            file_created_time,
            file_modified_time
        }
    }
}