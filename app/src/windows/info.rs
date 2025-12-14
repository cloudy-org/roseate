use std::{alloc, sync::Arc};

use cap::Cap;
use chrono::{DateTime, Local};
use egui::{Pos2, RichText, Ui, WidgetText};
use eframe::egui::{self, Response};

use crate::{image::image::Image, image_handler::ImageResource};

#[global_allocator]
static ALLOCATOR: Cap<alloc::System> = Cap::new(alloc::System, usize::max_value());

struct ImageInfoData {
    pub file_name: String,
    pub file_size: Option<f64>,
    pub image_created_time: Option<String>,
    pub file_modified_time: Option<String>,
}

impl ImageInfoData {
    pub fn new(image_handler_data: &ImageResource, image: &Image) -> Self {
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
        let mut image_created_time = None;
        let mut file_modified_time = None;

        if let Some(metadata) = file_metadata {
            let date_format = "%d/%m/%Y %H:%M %p";

            // TODO: prioritize using time picture was taken from EXIF tag instead of file created date.
            image_created_time = match metadata.created() {
                Ok(time) => {
                    let datetime: DateTime<Local> = time.into();
                    Some(datetime.format(date_format).to_string())
                },
                Err(error) => {
                    log::warn!("Failed to retrieve image file created date! Error: {}", error);

                    None
                },
            };

            file_modified_time = match metadata.modified() {
                Ok(time) => {
                    let datetime: DateTime<Local> = time.into();
                    Some(datetime.format(date_format).to_string())
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
            image_created_time,
            file_modified_time
        }
    }
}

pub struct ImageInfoWindow {
    data: Option<ImageInfoData>,
}

impl ImageInfoWindow {
    pub fn new() -> Self {
        Self {
            data: None,
        }
    }

    fn show_image_info_grid(
        ui: &mut Ui,
        image_info_data: &ImageInfoData,
        image: &Image,
        max_grid_width: f32,
        soon_text: Arc<RichText>,
        show_extra: bool
    ) {
        egui::Grid::new("base_image_info_grid")
            .striped(true)
            .max_col_width(max_grid_width)
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

                let created_hint = "Shows the date and time the image was taken or created \
                    according to your filesystem. (WARNING: EXIF tags are not used YET, so creation \
                    date is NOT accurate!)";

                ui.label("Created:").on_hover_text(created_hint);
                ui.label(
                    match &image_info_data.image_created_time {
                        Some(time_string) => RichText::new(time_string),
                        None => RichText::new("Unknown").weak(),
                    }
                ).on_hover_text(created_hint);
                ui.end_row();

                if show_extra {
                    ui.label("File Modified:");
                    ui.label(
                        match &image_info_data.file_modified_time {
                            Some(time_string) => RichText::new(time_string),
                            None => RichText::new("Unknown").weak(),
                        }
                    );
                    ui.end_row();
                }

                ui.label("File size:");
                ui.label(
                    match image_info_data.file_size {
                        Some(size) => RichText::new(re_format::format_bytes(size)),
                        None => RichText::new("Unknown").weak(),
                    }
                );
                ui.end_row();

                if show_extra {
                    ui.label("Camera:"); // 📷
                    ui.label(soon_text.clone());
                    ui.end_row();

                    ui.label("ISO:");
                    ui.label(soon_text.clone());
                    ui.end_row();

                    ui.label("Aperture:"); // ƒ
                    ui.label(soon_text.clone());
                    ui.end_row();

                    ui.label("Focal Length:");
                    ui.label(soon_text.clone());
                    ui.end_row();

                    ui.label("Exposure Time:");
                    ui.label(soon_text.clone());
                    ui.end_row();
                }
            });
    }

    fn show_misc_info_grid(
        ui: &mut Ui,
        image_info_data: &ImageInfoData,
        image: &Image,
        max_grid_width: f32,
        soon_text: Arc<RichText>,
        app_memory_allocated: f64
    ) {
        egui::Grid::new("misc_image_info_grid")
            .max_col_width(max_grid_width)
            .striped(false)
            .show(ui, |ui| {
                let mem_allocation_hint = "How much memory in total the entire app has been allocated.";

                ui.label("Mem Alloc:").on_hover_text(mem_allocation_hint);
                ui.label(
                    RichText::new(re_format::format_bytes(app_memory_allocated))
                ).on_hover_text(mem_allocation_hint);
                ui.end_row();

                let mem_allocation_by_image_hint = "How much memory has been allocated to display the image.";

                ui.label("Mem Alloc (by image):").on_hover_text(mem_allocation_by_image_hint);
                ui.label(soon_text.clone()).on_hover_text(mem_allocation_by_image_hint);
                ui.end_row();
            });
    }

    pub fn show(&mut self, ui: &Ui, image_handler_data: &ImageResource, image: &Image, show_extra: bool) -> Response {
        let image_info_data = self.data.get_or_insert_with(
            || ImageInfoData::new(image_handler_data, image)
        );

        let main_frame = egui::Frame::group(&ui.style())
            .inner_margin(8.0);

        let window = egui::Window::new(
            WidgetText::RichText(
                match show_extra {
                    false => RichText::new("ℹ Image Info"),
                    true => RichText::new("ℹ Image Info (Extra)"),
                }.size(15.0).into()
            )
        );

        window.default_pos(Pos2::new(200.0, 200.0))
            .min_width(150.0)
            .max_width(300.0)
            .resizable(false)
            .fade_in(false)
            .fade_out(false)
            .show(ui.ctx(), |ui| {
                // let available_width = ui.available_width();

                // let should_stack = match self.grid_width_used {
                //     Some(grid_width_used) => available_width < grid_width_used + 20.0,
                //     None => true
                // };

                // let main_layout = match should_stack {
                //     true => Layout::top_down(egui::Align::Min),
                //     false => Layout::left_to_right(egui::Align::Center)
                // };

                // let grid_width = (self.window_width.unwrap_or(available_width) / match should_stack {true => 2.0, false => 4.0}).min(200.0);

                main_frame.show(ui, |ui| {
                    ui.shrink_height_to_current();

                    ui.horizontal(|ui| {
                        let app_memory_allocated = ALLOCATOR.allocated();

                        let soon_text = Arc::new(
                            RichText::new("Coming Soon...").weak()
                        );

                        match show_extra {
                            true => {
                                // NOTE: ImageHandlerData::EguiImage doesn't work for some reason.
                                // ImageHandlerData::EguiImage will be entirely removed sometime anyways (https://github.com/cloudy-org/roseate/issues/89).
                                if let ImageResource::Texture(texture_handle) = image_handler_data {
                                    ui.add(
                                        egui::Image::from_texture(texture_handle)
                                            .max_height(100.0)
                                    );

                                    ui.add(egui::Separator::default().grow(3.0));
                                }

                                ui.vertical(|ui| {
                                    Self::show_image_info_grid(
                                        ui,
                                        image_info_data,
                                        image,
                                        180.0,
                                        soon_text.clone(),
                                        show_extra
                                    );

                                    ui.separator();

                                    Self::show_misc_info_grid(
                                        ui,
                                        image_info_data,
                                        image,
                                        180.0,
                                        soon_text.clone(),
                                        app_memory_allocated as f64
                                    );
                                });
                            },
                            false => {
                                ui.vertical(|ui| {
                                    Self::show_image_info_grid(
                                        ui, image_info_data, image, 160.0, soon_text, show_extra
                                    );
                                });
                            },
                        }
                    });
                });
            }).unwrap().response
    }
}