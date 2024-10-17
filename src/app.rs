use std::time::{Duration, Instant};

use cap::Cap;
use cirrus_theming::Theme;
use std::alloc;
use rdev::display_size;
use eframe::egui::{self, pos2, Color32, ImageSource, Key, Margin, Rect, Shadow, Style};

use crate::{image::{Image, ImageOptimization}, zooming_panning::ZoomingPanning};

#[global_allocator]
static ALLOCATOR: Cap<alloc::System> = Cap::new(alloc::System, usize::max_value());

pub struct Roseate {
    theme: Theme,
    image: Option<Image>,
    image_scale_factor: f32,
    zooming_panning: ZoomingPanning,
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
            zooming_panning: ZoomingPanning::new(),
            resize_timer: Some(Instant::now()),
            last_window_rect: Rect::NOTHING,
            image_loaded: false,
            show_info: false,
        }
    }

    fn show_info_box(&mut self, ctx: &egui::Context) {
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

        self.zooming_panning.handle_pan(ctx);
        self.zooming_panning.handle_zoom(ctx);

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

                    let (zoom_scaled_size, pan_image_position) = self.zooming_panning.get_transformation(
                        (scaled_image_width_animated as f32, scaled_image_height_animated as f32).into(), 
                        ui.max_rect().center()
                    );

                    let zooming_panning_rect = Rect::from_min_size(pan_image_position, zoom_scaled_size);

                    egui::Image::from_bytes(
                        format!("bytes://{}", image.image_path.to_string_lossy()), image.image_bytes.unwrap()
                    ).max_width(scaled_image_width_animated as f32)
                        .max_height(scaled_image_height_animated as f32)
                        .rounding(10.0)
                        .paint_at(ui, zooming_panning_rect);
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