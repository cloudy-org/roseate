use std::time::{Duration, Instant};

use rdev::display_size;
use cirrus_theming::Theme;
use eframe::egui::{self, Color32, ImageSource, Key, Margin, Rect};

use crate::{image::{Image, ImageOptimization}, zoom_pan::ZoomPan};

pub struct Roseate {
    pub theme: Theme,
    pub image: Option<Image>,
    image_scale_factor: f32,
    zoom_pan: ZoomPan,
    resize_timer: Option<Instant>,
    last_window_rect: Rect,
    image_loaded: bool,
    show_info: bool,
}

impl Roseate {
    pub fn new(image: Option<Image>, theme: Theme) -> Self {
        Self {
            image,
            theme,
            image_scale_factor: 1.0,
            zoom_pan: ZoomPan::new(),
            resize_timer: Some(Instant::now()),
            last_window_rect: Rect::NOTHING,
            image_loaded: false,
            show_info: false,
        }
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
        let central_panel_frame = egui::Frame {
            inner_margin: Margin::same(5.0),
            fill: Color32::from_hex(&self.theme.hex_code).unwrap(), // I mean... it should not fail... we know it's a valid hex colour...
            ..Default::default()
        };

        self.zoom_pan.handle_pan(ctx);
        self.zoom_pan.handle_zoom(ctx);

        egui::CentralPanel::default().frame(central_panel_frame).show(ctx, |ui| {
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
                    ui.add(
                        egui::Image::new(get_platform_rose_image()).max_width(130.0)
                    );
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

                    let (zoom_scaled_size, pan_image_position) = self.zoom_pan.get_transformation(
                        (scaled_image_width_animated as f32, scaled_image_height_animated as f32).into(), 
                        ui.max_rect().center()
                    );

                    let zoom_pan_rect = Rect::from_min_size(pan_image_position, zoom_scaled_size);

                    egui::Image::from_bytes(
                        format!("bytes://{}", image.image_path.to_string_lossy()), image.image_bytes.unwrap()
                    ).max_width(scaled_image_width_animated as f32)
                        .max_height(scaled_image_height_animated as f32)
                        .rounding(10.0)
                        .paint_at(ui, zoom_pan_rect);
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