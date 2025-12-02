use std::alloc;

use cap::Cap;
use egui::{Pos2, RichText, Ui, WidgetText};
use eframe::egui::{self, Margin, Response};

use crate::{image::image::Image};

#[global_allocator]
static ALLOCATOR: Cap<alloc::System> = Cap::new(alloc::System, usize::max_value());

pub struct InfoWindow {}

impl InfoWindow {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(&mut self, ui: &Ui, image: &Image) -> Response {
        let window = egui::Window::new(
            WidgetText::RichText(
                RichText::new("ℹ Info").size(15.0).into()
            )
        );

        window.default_pos(Pos2::new(200.0, 200.0))
            .resizable(false)
            .fade_in(false)
            .fade_out(false)
            .show(ui.ctx(), |ui| {
                let mem_allocated = ALLOCATOR.allocated();

                egui::Frame::group(&ui.ctx().style())
                    .inner_margin(Margin::same(1))
                    .show(ui, |ui| {
                        egui::Grid::new("info_box_grid")
                            .num_columns(3)
                            .spacing([20.0, 4.0])
                            .striped(true)
                            .max_col_width(130.0)
                            .show(ui, |ui| {
                                let image_metadata = image.image_path.metadata().expect(
                                    "Failed to retrieve file metadata!"
                                );

                                ui.label("Name:");
                                ui.label(
                                    image.image_path.file_name().expect("Failed to retrieve image name from path!").to_string_lossy()
                                );
                                ui.end_row();

                                ui.label("Dimensions: ");
                                ui.label(
                                    format!(
                                        "{}x{}", image.image_size.0, image.image_size.1
                                    )
                                );
                                ui.end_row();

                                ui.label("File size: ");
                                ui.label(
                                    format!(
                                        "{}", re_format::format_bytes(image_metadata.len() as f64)
                                    )
                                );
                                ui.end_row();
                            });
                    }
                );

                ui.add_space(3.0);
                ui.label(format!(
                        "Memory Allocated: {}",
                        re_format::format_bytes(mem_allocated as f64)
                ));
            }).unwrap().response
    }
}