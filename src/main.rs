#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use std::{env, path::Path, time::{Duration, Instant}};

use image::Image;
use eframe::egui::{self, ImageSource, Rect};

mod image;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    let cli_args: Vec<String> = env::args().collect();
    let image_path = cli_args.get(1);

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
            Ok(Box::new(Roseate::new(image)))
        }),
    )
}

struct Roseate {
    image: Option<Image>,
    image_scale_factor: f32,
    resize_timer: Option<Instant>,
    last_window_rect: Rect
}

impl Roseate {
    fn new(image: Option<Image>) -> Self {
        Self {
            image,
            image_scale_factor: 1.0,
            resize_timer: Some(Instant::now()),
            last_window_rect: Rect::NOTHING
        }
    }

    fn scale_image_on_window_resize(&mut self, window_rect: &Rect) {
        if let Some(timer) = self.resize_timer {
            // If the timer has expired (no new resize events)
            if timer.elapsed() >= Duration::from_millis(500) {
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

            if window_rect.width() != self.last_window_rect.width() && window_rect.height() != self.last_window_rect.height() {
                self.resize_timer = Some(Instant::now());
                self.last_window_rect = window_rect;
            }

            if self.image.is_none() {
                ui.centered_and_justified(|ui| {
                    ui.add(egui::Image::new(get_platform_rose_image()).max_width(130.0));
                });

                return;
            }

            self.scale_image_on_window_resize(&window_rect);

            let image = self.image.as_ref().unwrap(); // We can assume the image exists if the scale image function returns true.

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
                            format!("bytes://{}", image.image_path), image.image_bytes.clone()
                        ).max_width(scaled_image_width_animated as f32).max_height(scaled_image_height_animated as f32).rounding(10.0)
                    );
                });
            });

            ctx.request_repaint_after_secs(1.0); // We need to request repaints just in 
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