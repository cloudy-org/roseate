use std::{alloc, fmt::{self, Formatter}, sync::Arc};

use cap::Cap;
use chrono::{DateTime, Local, NaiveDateTime};
use egui::{AtomExt, Color32, Label, Pos2, RichText, Stroke, TextureHandle, Ui, Vec2, WidgetText};
use eframe::egui::{self, Response};
use roseate_core::{decoded_image::DecodedImageInfo, metadata::ImageMetadata};

use crate::{image::Image, image_handler::{optimization::ImageOptimizations, resource::ImageResource}};

#[global_allocator]
static ALLOCATOR: Cap<alloc::System> = Cap::new(alloc::System, usize::max_value());

macro_rules! rich_text_or_unknown {
    ($fmt:literal, $opt:expr) => {
        match &$opt {
            Some(string) => RichText::new(format!($fmt, string)),
            None => RichText::new("Unknown").weak(),
        }
    };
}

struct ExpensiveData {
    pub file_name: String,
    pub file_size: Option<f64>,
    pub file_relative_path: String,
    pub image_created_time: Option<String>,
    pub file_modified_time: Option<String>,
    pub memory_allocated_for_image: f64,
}

impl ExpensiveData {
    pub fn new(image_resource: &ImageResource, image_metadata: &ImageMetadata, image: &Image) -> Self {
        let path = &image.path;

        let file_name = path.file_name().unwrap().to_string_lossy().to_string();
        let file_relative_path = path.to_string_lossy().to_string();

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

        let date_format = "%d/%m/%Y %H:%M %p";

        if let Some(time) = &image_metadata.originally_created {
            log::debug!("originally created: {}", time);
            match NaiveDateTime::parse_from_str(time, "%Y-%m-%d %H:%M:%S") {
                Ok(datetime) => {
                    image_created_time = Some(datetime.format(date_format).to_string());
                },
                Err(err) => {
                    log::warn!("Failed to retrieve image file created date! Error: {}", err);
                }
            }
        }

        if let Some(metadata) = file_metadata {
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
            file_relative_path,
            image_created_time,
            file_modified_time,
            memory_allocated_for_image: match image_resource {
                ImageResource::Texture(texture_handle) => texture_handle.byte_size() as f64,
                ImageResource::AnimatedTexture(frames) => {
                    let mut size = 0;

                    for (texture_handler, _) in frames {
                        size += texture_handler.byte_size();
                    }

                    size as f64
                },
            }
        }
    }
}

pub struct ImageInfoWindow {
    data: Option<ExpensiveData>,
}

impl ImageInfoWindow {
    pub fn new() -> Self {
        Self {
            data: None,
        }
    }

    fn show_image_optimizations_grid(ui: &mut Ui, image_optimizations: &ImageOptimizations) {
        egui::Frame::default()
            .inner_margin(8)
            .corner_radius(8)
            .fill(Color32::BLACK.gamma_multiply(0.2))
            .show(ui, |ui| {

                egui::Grid::new("image_optimizations_grid")
                    .max_col_width(120.0)
                    .striped(false)
                    .show(ui, |ui| {
                        // I'm using let Some() because in the future
                        // I'll actually make use of the struct inside.

                        if let Some(_) = image_optimizations.monitor_downsampling {
                            ui_non_select_label(ui, "Monitor downsampling:");
                            ui.label("applied");
                            ui.end_row();
                        }

                        if let Some(_) = image_optimizations.dynamic_sampling {
                            ui_non_select_label(ui, "Dynamic sampling:");
                            ui.label("enabled");
                            ui.end_row();
                        }

                        if image_optimizations.consume_pixels_during_gpu_upload {
                            ui_non_select_label(ui, "Consume pixels during GPU upload:");
                            ui.label("enabled");
                            ui.end_row();
                        }

                        if let Some(multi_threaded_sampling) = &image_optimizations.multi_threaded_sampling {
                            ui_non_select_label(ui, "Multi threaded downsampling:");
                            ui.label(
                                match multi_threaded_sampling.number_of_threads {
                                    Some(threads) => format!("{} threads", threads),
                                    None => "auto".into(),
                                }
                            );
                            ui.end_row();
                        }
                    });

            });
    }

    fn show_image_info_grid(
        ui: &mut Ui,
        expensive_data: &ExpensiveData,
        image: &Image,
        decoded_image_info: &DecodedImageInfo,
        max_grid_width: f32,
        soon_text: Arc<RichText>,
        show_extra: bool
    ) {
        egui::Grid::new("base_image_info_grid")
            .striped(true)
            .max_col_width(max_grid_width)
            .show(ui, |ui| {
                ui_non_select_label(ui, "Name:");
                ui.label(&expensive_data.file_name)
                    .on_hover_text(&expensive_data.file_relative_path);
                ui.end_row();

                ui_non_select_label(ui, "Dimensions:");
                ui.label(
                    format!(
                        "{}x{}", image.size.0, image.size.1
                    )
                );
                ui.end_row();

                ui_non_select_label(ui, "Format:");
                ui.label(format!("{}", image.format));
                ui.end_row();

                if show_extra {
                    ui_non_select_label(ui, "Colour:");
                    ui.label(format!("{}", decoded_image_info.colour_type));
                    ui.end_row();
                }

                let created_hint = "Shows the date and time the image was taken or created \
                    according to your filesystem. (WARNING: EXIF tags are not used YET, so creation \
                    date is NOT accurate!)";

                ui_non_select_label(ui, "Created:").on_hover_text(created_hint);
                ui.label(
                    match &expensive_data.image_created_time {
                        Some(time_string) => RichText::new(time_string),
                        None => RichText::new("Unknown").weak(),
                    }
                ).on_hover_text(created_hint);
                ui.end_row();

                if show_extra {
                    ui_non_select_label(ui, "File Modified:");
                    ui.label(
                        match &expensive_data.file_modified_time {
                            Some(time_string) => RichText::new(time_string),
                            None => RichText::new("Unknown").weak(),
                        }
                    );
                    ui.end_row();
                }

                ui_non_select_label(ui, "File size:");
                ui.label(
                    match expensive_data.file_size {
                        Some(size) => RichText::new(re_format::format_bytes(size)),
                        None => RichText::new("Unknown").weak(),
                    }
                );
                ui.end_row();

                if show_extra {
                    ui_non_select_label(ui, "Camera:");
                    ui.label(rich_text_or_unknown!("{}", &decoded_image_info.metadata.model));
                    ui.end_row();

                    ui_non_select_label(ui, "ISO:");
                    ui.label(rich_text_or_unknown!("{}", &decoded_image_info.metadata.iso));
                    ui.end_row();

                    ui_non_select_label(ui, "Aperture:");
                    ui.label(rich_text_or_unknown!("ƒ/{}", &decoded_image_info.metadata.aperture));
                    ui.end_row();

                    ui_non_select_label(ui, "Focal Length:");
                    ui.label(rich_text_or_unknown!("{}mm", &decoded_image_info.metadata.focal_length));
                    ui.end_row();

                    ui_non_select_label(ui, "Exposure Time:");
                    ui.label(rich_text_or_unknown!("{}s", &decoded_image_info.metadata.exposure_time));
                    ui.end_row();
                }
            });
    }

    fn show_misc_info_grid(
        ui: &mut Ui,
        expensive_data: &ExpensiveData,
        image: &Image,
        decoded_image_info: &DecodedImageInfo,
        max_grid_width: f32,
        soon_text: Arc<RichText>,
        app_memory_allocated: f64,
    ) {
        egui::Grid::new("misc_image_info_grid")
            .max_col_width(max_grid_width)
            .striped(false)
            .show(ui, |ui| {
                let mem_allocation_hint = "How much memory has been allocated to the entire application \
                (this includes the decoded image, if it's still in memory).";

                ui_non_select_label(ui, "App Mem Alloc:")
                    .on_hover_text(mem_allocation_hint);
                ui.label(RichText::new(re_format::format_bytes(app_memory_allocated)))
                    .on_hover_text(mem_allocation_hint);
                ui.end_row();

                let mem_allocation_by_image_hint = "How much memory has been allocated to display the image on the GPU.";

                ui_non_select_label(ui, "Image Mem Alloc:")
                    .on_hover_text(mem_allocation_by_image_hint);
                ui.label(
                    RichText::new(re_format::format_bytes(
                        expensive_data.memory_allocated_for_image)
                    )
                ).on_hover_text(mem_allocation_by_image_hint);
                ui.end_row();
            });
    }

    pub fn show(
        &mut self,
        ui: &Ui,
        image_resource: &ImageResource,
        image_optimizations: &ImageOptimizations,
        image: &Image,
        decoded_info_image: &DecodedImageInfo,
        show_extra: bool
    ) -> Response {
        let image_info_data = self.data.get_or_insert_with(
            || ExpensiveData::new(image_resource, &decoded_info_image.metadata, image)
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
                                let texture_handle: Option<&TextureHandle> = match image_resource {
                                    ImageResource::Texture(texture_handle) => Some(texture_handle),
                                    ImageResource::AnimatedTexture(frames) => {
                                        frames.get(0)
                                            .and_then(
                                                |(texture_handle,_)| Some(texture_handle)
                                            )
                                    }
                                };

                                ui.vertical(|ui| {
                                    if let Some(texture) = texture_handle {

                                        ui.add(
                                            egui::Image::from_texture(texture)
                                                // 16 is the padding from
                                                // the image optimizations grid
                                                .max_size([200.0 + 16.0, 140.0].into())
                                                .corner_radius(8)
                                        );
                                    }

                                    ui.add_space(5.0);

                                    Self::show_image_optimizations_grid(ui, image_optimizations);
                                });

                                ui.add(egui::Separator::default().grow(4.0));

                                ui.vertical(|ui| {
                                    Self::show_image_info_grid(
                                        ui,
                                        image_info_data,
                                        image,
                                        decoded_info_image,
                                        180.0,
                                        soon_text.clone(),
                                        show_extra
                                    );

                                    ui.separator();

                                    Self::show_misc_info_grid(
                                        ui,
                                        image_info_data,
                                        image,
                                        decoded_info_image,
                                        180.0,
                                        soon_text.clone(),
                                        app_memory_allocated as f64,
                                    );
                                });
                            },
                            false => {
                                ui.vertical(|ui| {
                                    Self::show_image_info_grid(
                                        ui, image_info_data, image, decoded_info_image, 160.0, soon_text, show_extra
                                    );
                                });
                            },
                        }
                    });
                });
            }).unwrap().response
    }
}

fn ui_non_select_label(ui: &mut Ui, text: impl Into<WidgetText>) -> Response {
    ui.add(Label::new(text).selectable(false))
}
