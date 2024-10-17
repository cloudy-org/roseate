use std::alloc;

use cap::Cap;
use eframe::egui::{self, pos2, Color32, Margin, Shadow, Style};

use crate::app::Roseate;

#[global_allocator]
static ALLOCATOR: Cap<alloc::System> = Cap::new(alloc::System, usize::max_value());

impl Roseate {
    pub fn show_info_box(&mut self, ctx: &egui::Context) {

        let mut custom_frame = egui::Frame::window(&ctx.style());
        custom_frame.fill = Color32::from_hex(&self.theme.hex_code).unwrap().gamma_multiply(3.0);
        custom_frame.shadow = Shadow::NONE;

        egui::Window::new(
            egui::WidgetText::RichText(
                egui::RichText::new("ℹ Info").size(15.0)
            )
        )
            .default_pos(pos2(200.0, 200.0))
            .title_bar(true)
            .resizable(false)
            .frame(custom_frame)
            .show(ctx, |ui| {
                let mem_allocated = ALLOCATOR.allocated();

                egui::Frame::group(&Style::default()).inner_margin(Margin::same(1.0)).show(
                    ui, |ui| {
                        egui::Grid::new("info_box_grid")
                        .num_columns(2)
                        .spacing([20.0, 4.0])
                        .striped(true)
                        .max_col_width(130.0)
                        .show(ui, |ui| {
                            if self.image.is_some() {
                                let image = self.image.as_ref().unwrap(); // safe to unwrap as we know this is Some().

                                ui.label("Name:");
                                ui.label(
                                    image.image_path.file_name().expect("Failed to retrieve image name from path!").to_string_lossy()
                                );
                                ui.end_row();

                                ui.label("Dimensions: ");
                                ui.label(
                                    format!(
                                        "{}x{}", image.image_size.width, image.image_size.height
                                    )
                                );
                                ui.end_row();
                            }
                        });
                    }
                );

                ui.add_space(3.0);
                ui.label(format!(
                        "Memory Allocated: {}",
                        re_format::format_bytes(mem_allocated as f64)
                ));
            });
    }
}