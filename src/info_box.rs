use std::alloc;

use cap::Cap;
use egui_notify::ToastLevel;
use eframe::egui::{self, pos2, Key, Margin, Response};

use crate::{config::Config, image::Image, toasts::ToastsManager};

#[global_allocator]
static ALLOCATOR: Cap<alloc::System> = Cap::new(alloc::System, usize::max_value());

pub struct InfoBox {
    pub show: bool,
    image: Option<Image>,
    toggle_key: Key,
    pub response: Option<Response>
}

impl InfoBox {
    pub fn new(config: &Config, toasts: &mut ToastsManager) -> Self {
        let config_key = match Key::from_name(&config.keybinds.info_box.toggle) {
            Some(key) => key,
            None => {
                toasts.toast_and_log(
                    "The key bind set for 'info_box.toggle' is invalid! Defaulting to `I`.".into(), 
                    ToastLevel::Error
                );

                Key::I
            },
        };

        Self {
            show: false,
            image: None,
            toggle_key: config_key,
            response: None
        }
    }

    pub fn init(&mut self, image: &Option<Image>) {
        self.image = image.clone();
    }

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
            let response = egui::Window::new(
                egui::WidgetText::RichText(
                    egui::RichText::new("ℹ Info").size(15.0)
                )
            )
                .default_pos(pos2(200.0, 200.0))
                .resizable(false)
                .fade_in(false)
                .fade_out(false)
                .show(ctx, |ui| {
                    let mem_allocated = ALLOCATOR.allocated();

                    egui::Frame::group(&ctx.style()).inner_margin(Margin::same(1.0)).show(
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

            self.response = Some(response.unwrap().response);
        }
    }
}