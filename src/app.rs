use std::{time::Duration, vec};

use egui_notify::Toasts;
use cirrus_theming::Theme;
use eframe::egui::{self, Align, Color32, Context, CursorIcon, Frame, ImageSource, Layout, Margin, Rect, Shadow, Stroke, Style, TextStyle, Vec2};

use crate::{error, files, image::Image, image_loader::ImageLoader, info_box::InfoBox, magnification_panel::MagnificationPanel, window_scaling::WindowScaling, zoom_pan::ZoomPan};

pub struct Roseate {
    theme: Theme,
    image: Option<Image>,
    toasts: Toasts,
    zoom_pan: ZoomPan,
    info_box: InfoBox,
    magnification_panel: MagnificationPanel,
    window_scaling: WindowScaling,
    last_window_rect: Rect,
    image_loaded: bool,
    image_loader: ImageLoader,
    last_window_rect: Rect
    dropped_files: Vec<egui::DroppedFile>,
}

impl Roseate {
    pub fn new(image: Option<Image>, theme: Theme, toasts: Toasts) -> Self {
        let mut image_loader = ImageLoader::new();

        if image.is_some() {
            image_loader.load_image(&mut image.clone().unwrap(), false);
        }

        Self {
            image,
            theme,
            toasts: toasts,
            zoom_pan: ZoomPan::new(),
            info_box: InfoBox::new(),
            magnification_panel: MagnificationPanel::new(),
            window_scaling: WindowScaling::new(),
            last_window_rect: Rect::NOTHING,
            image_loaded: false,
            dropped_files: vec![]
            image_loader: image_loader,
            last_window_rect: Rect::NOTHING,
            dropped_files: Vec<egui::DroppedFile>,
        }
    }

    fn set_app_style(&self, ctx: &Context) {
        // #1d0a0a # dark mode secondary colour for roseate
        let mut custom_style = Style {
            override_text_style: Some(TextStyle::Monospace),
            ..Default::default()
        };

        // TODO: override more default   
        // colours here with colours from our theme.

        // Background colour styling.
        custom_style.visuals.panel_fill = Color32::from_hex(
            &self.theme.primary_colour.hex_code
        ).unwrap();

        // Window styling.
        custom_style.visuals.window_highlight_topmost = false;

        custom_style.visuals.window_fill = Color32::from_hex(
            &self.theme.secondary_colour.hex_code
        ).unwrap();
        custom_style.visuals.window_stroke = Stroke::new(
            1.0,
            Color32::from_hex(&self.theme.third_colour.hex_code).unwrap()
        );
        custom_style.visuals.window_shadow = Shadow::NONE;

        // Text styling.
        custom_style.visuals.override_text_color = Some(
            Color32::from_hex(
                match self.theme.is_dark {
                    true => "#b5b5b5",
                    false => "#3b3b3b"
                }
            ).unwrap()
        );

        ctx.set_style(custom_style);
    }
}

impl eframe::App for Roseate {

    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.set_app_style(ctx);

        self.info_box.init(&self.image);

        self.info_box.handle_input(ctx);
        self.zoom_pan.handle_zoom_input(ctx);
        self.zoom_pan.handle_reset_input(ctx);
        self.magnification_panel.handle_input(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            let window_rect = ctx.input(|i: &egui::InputState| i.screen_rect());

            if window_rect.width() != self.last_window_rect.width() || window_rect.height() != self.last_window_rect.height() {
                if !self.zoom_pan.has_been_messed_with() {
                    self.window_scaling.schedule_scale_image_to_window_size();
                    self.last_window_rect = window_rect;
                }
            }

            self.toasts.show(ctx);

            if self.image.is_none() {            
                // Collect dropped files
                ctx.input(|i| {
                    if !i.raw.dropped_files.is_empty() {
                        self.dropped_files.clone_from(&i.raw.dropped_files);
                    }
                });

                if !self.dropped_files.is_empty() {
                    let path = self.dropped_files.first().unwrap().clone().path.unwrap(); // gotta love rust
                    let image = Image::from_path(&path);

                    self.image = Some(image.clone());
                    self.info_box = InfoBox::new(Some(image.clone()), self.theme.clone());
                }

                if !ctx.input(|i| i.raw.hovered_files.is_empty()) {
                    ui.centered_and_justified(|ui| {
                        let osaka_width: f32 = 130.0;
                        let osaka = egui::include_image!("../assets/osaka.png");
    
                        egui::Frame::default()
                            .outer_margin(
                                Margin::symmetric(
                                    (window_rect.width() / 2.0) - osaka_width / 2.0, 
                                    (window_rect.height() / 2.0) - osaka_width / 2.0
                                )
                            )
                            .show(ui, |ui| {
                                ui.add(
                                    egui::Image::new(osaka)
                                    .max_width(osaka_width)
                                );

                                ui.label("Drop your file.");
                            }
                        );
                    });
                }
                
                ui.centered_and_justified(|ui| {
                    let rose_width: f32 = 130.0;

                    egui::Frame::default()
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
                                    Ok(mut image) => {
                                        self.image = Some(image.clone());
                                        // TODO: Use config's value for lazy load instead.
                                        self.image_loader.load_image(&mut image, true);
                                    },
                                    Err(error) => {
                                        error::log_and_toast(error.into(), &mut self.toasts)
                                            .duration(Some(Duration::from_secs(5)));
                                    },
                                }
                            }
                        }
                    );
                });

                return; // don't do anything else, you know, like stop right there bitch
            }

            self.info_box.update(ctx);
            self.zoom_pan.update(ctx);
            self.image_loader.update(&mut self.toasts);
            self.magnification_panel.update(ctx, &mut self.zoom_pan);

            let image = self.image.clone().unwrap();

            if self.image_loader.image_loaded {
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
                        format!(
                            "bytes://{}", image.image_path.to_string_lossy()
                        ),
                        // we can unwrap because we know the bytes exist thanks to 'self.image_loader.image_loaded'.
                        image.image_bytes.lock().unwrap().clone().unwrap()
                    ).rounding(10.0)
                        .paint_at(ui, zoom_pan_rect);
    

                    self.zoom_pan.handle_pan_input(ctx, &response, self.info_box.response.as_ref());
                });

                // TODO:

                // We must update the WindowScaling with the window size AFTER
                // the image has loaded to maintain that smooth scaling animation.
                self.window_scaling.update(&window_rect, &image.image_size);

                ctx.request_repaint_after_secs(0.5); // We need to request repaints just in 
                // just in case one doesn't happen when the window is resized in a certain circumstance 
                // (i.e. the user maximizes the window and doesn't interact with it). I'm not sure how else we can fix it.
            }
        });

        // This is deliberately placed after the central panel so the central panel 
        // can take up all the space essentially ignoring the space this panel would otherwise take.
        // Check out the egui docs for more clarification: https://docs.rs/egui/0.29.1/egui/containers/panel/struct.CentralPanel.html
        egui::TopBottomPanel::bottom("status_bar")
            .show_separator_line(false)
            .frame(
                Frame::none()
                    .outer_margin(Margin {left: 10.0, bottom: 7.0, ..Default::default()})
            ).show(ctx, |ui| {
                if let Some(loading) = &self.image_loader.image_loading {
                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                        ui.add(
                            egui::Spinner::new()
                                .color(Color32::from_hex("#e05f78").unwrap()) // NOTE: This should be the default accent colour.
                                .size(20.0)
                        );
    
                        if let Some(message) = &loading.message {
                            ui.label(message);
                        }
                    });
                }
            }
        );

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