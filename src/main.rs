#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use std::{env, path::Path, time::{Duration, Instant}};

use cap::Cap;
use std::alloc;
use log::debug;
use rdev::display_size;
use image::{Image, ImageOptimization};
use eframe::egui::{self, pos2, ImageSource, Key, Margin, Rect, RichText, Style};

#[global_allocator]
static ALLOCATOR: Cap<alloc::System> = Cap::new(alloc::System, usize::max_value());

mod image;

fn main() -> eframe::Result {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    let cli_args: Vec<String> = env::args().collect();
    let image_path = cli_args.get(1);

    if image_path.is_some() {
        debug!("Image '{}' loading from path...", image_path.unwrap());
    }

    let image = match image_path {
        Some(path) => {
            let path = Path::new(path);
            Some(Image::from_path(path))
        },
        None => None
    };

    eframe::run_native(
        "Roseate",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            debug!("image loaded from path!");
            Ok(Box::new(Roseate::new(image)))
        }),
    )
}

struct Roseate {
    image: Option<Image>,
    image_scale_factor: f32,
    resize_timer: Option<Instant>,
    last_window_rect: Rect,
    image_loaded: bool,
    show_info: bool,
}

impl Roseate {
    fn new(image: Option<Image>) -> Self {
        Self {
            image,
            image_scale_factor: 1.0,
            resize_timer: Some(Instant::now()),
            last_window_rect: Rect::NOTHING,
            image_loaded: false,
            show_info: false,
        }
    }

    fn show_info_box(&mut self, ctx: &egui::Context) {
        egui::Window::new(
            egui::WidgetText::RichText(
                egui::RichText::new("ℹ Info").size(15.0)
            )
        )
            .default_pos(pos2(200.0, 200.0))
            .title_bar(true)
            .show(ctx, |ui| {
                let mem_allocated = ALLOCATOR.allocated();

                ui.vertical_centered(|ui| {

                    if self.image.is_some() {
                        let image = self.image.as_ref().unwrap(); // safe to unwrap as we know this is Some().

                        egui::Frame::group(&Style::default()).inner_margin(Margin::same(1.0)).show(
                            ui, |ui| {
                                ui.heading(RichText::new("Image").underline().size(15.0));
                                ui.add_space(1.0);
                                ui.label(
                                    format!(
                                        "Dimensions: {}x{}", image.image_size.width, image.image_size.height
                                    )
                                );
                                ui.label(
                                    format!(
                                        "Name: {}",
                                        image.image_path.file_name().expect("Failed to retrieve image name from path!").to_string_lossy()
                                    )
                                );
                            }
                        );

                        ui.add_space(3.0);
                    }

                    ui.label(format!(
                        "Memory Allocated: {}",
                        re_format::format_bytes(mem_allocated as f64)
                    ));
                });
            });
    }

    fn scale_image_on_window_resize(&mut self, window_rect: &Rect) {
        if let Some(timer) = self.resize_timer {
            // If the timer has expired (no new resize events)
            if timer.elapsed() >= Duration::from_millis(300) {
                // Reset the timer
                self.resize_timer = None;

                let image = self.image.as_ref().unwrap(); // we can assume this as we checked in the first line.

                let scale_x = window_rect.width() / image.image_size.width as f32;
                let scale_y = window_rect.height() / image.image_size.height as f32;

                let scale_factor = scale_x.min(scale_y); // Scale uniformly.

                // Make sure scale_factor doesn't exceed the original size (1).
                self.image_scale_factor = scale_factor.min(1.0);
            }
        }
    }
}

impl eframe::App for Roseate {

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        egui::CentralPanel::default().show(ctx, |ui| {
            let window_rect = ctx.input(|i: &egui::InputState| i.screen_rect());

            if window_rect.width() != self.last_window_rect.width() || window_rect.height() != self.last_window_rect.height() {
                self.resize_timer = Some(Instant::now());
                self.last_window_rect = window_rect;
            }

            if ctx.input(|i| i.key_pressed(Key::I)) {
                if self.show_info == true {
                    self.show_info = false;
                } else {
                    self.show_info = true;
                }
            }

            if self.image.is_none() {
                ui.centered_and_justified(|ui| {
                    ui.add(egui::Image::new(get_platform_rose_image()).max_width(130.0));
                });

                return;
            }

            if self.show_info {
                self.show_info_box(ctx);
            }

            if !self.image_loaded {
                let mutable_image = self.image.as_mut().unwrap();

                let (width, height) = display_size().expect("Failed to get monitor size!");

                let mut optimizations = Vec::new();

                if mutable_image.image_size.width > width as usize && mutable_image.image_size.height > height as usize {
                    optimizations.push(ImageOptimization::Downsample(width as u32, height as u32));
                }

                mutable_image.load_image(&optimizations);

                self.image_loaded = true;
            }

            let image = self.image.clone().unwrap();

            self.scale_image_on_window_resize(&window_rect);

            ui.centered_and_justified(|ui| {
                egui::ScrollArea::both().show(ui, |ui| {
                    let scaled_image_width = image.image_size.width as f32 * self.image_scale_factor;
                    let scaled_image_height = image.image_size.height as f32 * self.image_scale_factor;

                    let scaled_image_width_animated = egui_animation::animate_eased(
                        ctx, "image_scale_width", scaled_image_width, 1.5, simple_easing::cubic_in_out
                    ) as u32;
                    let scaled_image_height_animated = egui_animation::animate_eased(
                        ctx, "image_scale_height", scaled_image_height, 1.5, simple_easing::cubic_in_out
                    ) as u32;

                    ui.add(
                        egui::Image::from_bytes(
                            format!("bytes://{}", image.image_path.to_string_lossy()), image.image_bytes.unwrap()
                        ).max_width(scaled_image_width_animated as f32).max_height(scaled_image_height_animated as f32).rounding(10.0)
                    );
                });
            });

            ctx.request_repaint_after_secs(0.5); // We need to request repaints just in 
            // just in case one doesn't happen when the window is resized in a certain circumstance 
            // (i.e. the user maximizes the window and doesn't interact with it). I'm not sure how else we can fix it.
        });

    }

}

fn get_platform_rose_image<'a>() -> ImageSource<'a> {
    if cfg!(target_os = "windows") {
        return egui::include_image!("../assets/rose_emojis/microsoft.png");
    } else if cfg!(target_os = "macos") {
        return egui::include_image!("../assets/rose_emojis/apple.png");
    }

    return egui::include_image!("../assets/rose_emojis/google_noto.png");
}