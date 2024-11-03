use std::time::Duration;

use cirrus_theming::Theme;
use eframe::egui::{self, Color32, CursorIcon, ImageSource, Margin, Rect, Stroke, Vec2};
use egui_notify::Toasts;

use crate::{error, files, image::{apply_image_optimizations, Image}, info_box::InfoBox, window_scaling::WindowScaling, zoom_pan::ZoomPan};

pub struct Roseate {
    theme: Theme,
    image: Option<Image>,
    toasts: Toasts,
    zoom_pan: ZoomPan,
    info_box: InfoBox,
    window_scaling: WindowScaling,
    last_window_rect: Rect,
    image_loaded: bool
}

impl Roseate {
    pub fn new(image: Option<Image>, theme: Theme, toasts: Toasts) -> Self {
        let (ib_image, ib_theme) = (image.clone(), theme.clone());

        Self {
            image,
            theme,
            toasts: toasts,
            zoom_pan: ZoomPan::new(),
            info_box: InfoBox::new(ib_image, ib_theme),
            window_scaling: WindowScaling::new(),
            last_window_rect: Rect::NOTHING,
            image_loaded: false
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

        self.info_box.handle_input(ctx);
        self.zoom_pan.handle_zoom_input(ctx);
        self.zoom_pan.handle_reset_input(ctx);

        egui::CentralPanel::default().frame(central_panel_frame).show(ctx, |ui| {
            let window_rect = ctx.input(|i: &egui::InputState| i.screen_rect());

            if window_rect.width() != self.last_window_rect.width() || window_rect.height() != self.last_window_rect.height() {
                if !self.zoom_pan.has_been_messed_with() {
                    self.window_scaling.schedule_scale_image_to_window_size();
                    self.last_window_rect = window_rect;
                }
            }

            self.toasts.show(ctx);

            if self.image.is_none() {
                ui.centered_and_justified(|ui| {
                    let rose_width: f32 = 130.0;

                    egui::Frame::default()
                        .stroke(Stroke::default())
                        .outer_margin(
                            // I adjust the margin as it's the only way I know to 
                            // narrow down the interactive part (clickable part) of the rose image.
                            Margin::symmetric(
                                (window_rect.width() / 2.0) - rose_width / 2.0, 
                                (window_rect.height() / 2.0) - rose_width / 2.0
                            )
                        )
                        .show(ui, |ui| {
                            let response = ui.add(
                                egui::Image::new(get_platform_rose_image())
                                    .max_width(rose_width)
                                    .sense(egui::Sense::click())
                            );

                            response.clone().on_hover_cursor(CursorIcon::PointingHand);

                            if response.clicked() {
                                let image_result = files::select_image();

                                match image_result {
                                    Ok(image) => {
                                        // TODO: Need to improve this. Possibly by introducing an init function for info box .
                                        self.image = Some(image.clone());
                                        self.info_box = InfoBox::new(Some(image.clone()), self.theme.clone());
                                    },
                                    Err(error) => {
                                        error::log_and_toast(error, &mut self.toasts)
                                            .duration(Some(Duration::from_secs(5)));
                                    },
                                }
                            }
                        }
                    );
                });

                return;
            }

            self.info_box.update(ctx);
            self.zoom_pan.update(ctx);

            if !self.image_loaded {
                let mutable_image = self.image.as_mut().unwrap();

                let mut optimizations = Vec::new();
                optimizations = apply_image_optimizations(optimizations, &mutable_image.image_size);

                mutable_image.load_image(&optimizations);

                self.image_loaded = true;
            }

            let image = self.image.clone().unwrap();

            self.window_scaling.update(&window_rect, &image.image_size);

            ui.centered_and_justified(|ui| {
                let scaled_image_size = self.window_scaling.relative_image_size(
                    Vec2::new(image.image_size.width as f32, image.image_size.height as f32)
                );

                if self.zoom_pan.is_pan_out_of_bounds(scaled_image_size) {
                    self.zoom_pan.schedule_pan_reset(Duration::from_millis(300));
                };

                // NOTE: umm do we move this to window scaling... *probably* if we 
                // want to stay consistent with zoom_pan but this isn't important right now.
                let scaled_image_width_animated = egui_animation::animate_eased(
                    ctx, "image_scale_width", scaled_image_size.x, 1.5, simple_easing::cubic_in_out
                ) as u32 as f32;
                let scaled_image_height_animated = egui_animation::animate_eased(
                    ctx, "image_scale_height", scaled_image_size.y, 1.5, simple_easing::cubic_in_out
                ) as u32 as f32;

                let scaled_image_size = Vec2::new(scaled_image_width_animated, scaled_image_height_animated);

                let zoom_scaled_image_size = self.zoom_pan.relative_image_size(scaled_image_size);
                let image_position = ui.max_rect().center() - zoom_scaled_image_size * 0.5 + self.zoom_pan.pan_offset;

                let zoom_pan_rect = Rect::from_min_size(image_position, zoom_scaled_image_size);

                let response = ui.allocate_rect(zoom_pan_rect, egui::Sense::hover());

                egui::Image::from_bytes(
                    format!("bytes://{}", image.image_path.to_string_lossy()), image.image_bytes.unwrap()
                ).rounding(10.0)
                    .paint_at(ui, zoom_pan_rect);

                self.zoom_pan.handle_pan_input(ctx, &response, self.info_box.response.as_ref());
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