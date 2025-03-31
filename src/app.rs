use std::time::Duration;

use cirrus_theming::v1::{Colour, Theme};
use eframe::egui::{self, Align, Color32, Context, CursorIcon, Frame, Layout, Margin, Rect, Stroke, Vec2};
use egui_notify::ToastLevel;

use crate::{config::config::Config, files, image_handler::ImageHandler, magnification_panel::MagnificationPanel, monitor_size::MonitorSize, notifier::NotifierAPI, window_scaling::WindowScaling, windows::{about::AboutWindow, info::InfoWindow}, zoom_pan::ZoomPan};

pub struct Roseate<'a> {
    theme: Theme,
    zoom_pan: ZoomPan,
    info_box: InfoWindow,
    about_box: AboutWindow<'a>,
    notifier: NotifierAPI,
    magnification_panel: MagnificationPanel,
    window_scaling: WindowScaling,
    last_window_rect: Rect,
    image_handler: ImageHandler,
    monitor_size: MonitorSize,
    config: Config,
}

impl<'a> Roseate<'a> {
    pub fn new(mut image_handler: ImageHandler, monitor_size: MonitorSize, mut notifier: NotifierAPI, theme: Theme, config: Config) -> Self {
        if image_handler.image.is_some() {
            image_handler.load_image(
                config.image.loading.initial.lazy_loading,
                false,
                &mut notifier,
                &monitor_size,
                config.misc.experimental.use_fast_roseate_backend
            );
        }

        let zoom_pan = ZoomPan::new(&config, &mut notifier);
        let info_box = InfoWindow::new(&config, &mut notifier);
        let about_box = AboutWindow::new(&config, &mut notifier);
        let magnification_panel = MagnificationPanel::new(&config, &mut notifier);

        Self {
            theme,
            notifier,
            zoom_pan,
            info_box,
            about_box,
            magnification_panel,
            window_scaling: WindowScaling::new(&config),
            last_window_rect: Rect::NOTHING,
            monitor_size,
            image_handler,
            config,
        }
    }

    fn draw_dotted_line(&self, ui: &egui::Painter, pos: &[egui::Pos2]) {
        ui.add(
            egui::Shape::dashed_line(
                pos, 
                Stroke {
                    width: 2.0,
                    color: Color32::from_hex(
                        &self.theme.accent_colour.hex_code
                    ).unwrap()
                },
                10.0, 
                10.0
            )
        );
    }
}

impl eframe::App for Roseate<'_> {

    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.info_box.init(&self.image_handler.image);
        self.info_box.handle_input(ctx);

        self.zoom_pan.handle_reset_input(ctx);
        self.magnification_panel.handle_input(ctx);
        self.about_box.handle_input(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            let window_rect = ctx.input(|i: &egui::InputState| i.screen_rect());

            if window_rect.width() != self.last_window_rect.width() || window_rect.height() != self.last_window_rect.height() {
                if !self.zoom_pan.has_been_messed_with() {
                    self.window_scaling.schedule_scale_image_to_window_size();
                    self.last_window_rect = window_rect;
                }
            }

            self.notifier.update(ctx);
            self.monitor_size.update(ctx, &mut self.notifier);
            self.about_box.update(ctx); // we update this box here because we want 
            // the about box is to be toggleable even without an image.

            if self.image_handler.image.is_none() {
                // Collect dropped files.
                ctx.input(|i| {
                    let dropped_files = &i.raw.dropped_files;

                    if !dropped_files.is_empty() {
                        let path = dropped_files.first().unwrap()
                            .path
                            .as_ref()
                            .unwrap(); // gotta love rust ~ ananas

                        let result = self.image_handler.init_image(path);

                        if let Err(error) = result {
                            self.notifier.toasts.lock().unwrap().toast_and_log(
                                error.into(), ToastLevel::Error
                            );
                            return;
                        }

                        self.image_handler.load_image(
                            true, 
                            false,
                            &mut self.notifier,
                            &self.monitor_size,
                            self.config.misc.experimental.use_fast_roseate_backend
                        );
                    }
                });

                ui.centered_and_justified(|ui| {
                    let rose_width: f32 = 145.0;
                    let file_is_hovering = !ctx.input(|i| i.raw.hovered_files.is_empty());

                    let mut rose_rect = Rect::NOTHING;

                    egui::Frame::default()
                        .outer_margin(
                            // I adjust the margin as it's the only way I know to 
                            // narrow down the interactive part (clickable part) of the rose image.
                            Margin::symmetric(
                                // NOTE: width and height of rose are the same.
                                (window_rect.width() / 2.0) - rose_width / 2.0, 
                                (window_rect.height() / 2.0) - rose_width / 2.0
                            )
                        )
                        .show(ui, |ui| {
                            let rose_response = ui.add(
                                egui::Image::new(files::get_platform_rose_image())
                                    .max_width(rose_width)
                                    .sense(egui::Sense::click())
                            );

                            rose_rect = rose_response.rect;

                            if file_is_hovering {
                                ui.label("You're about to drop a file.");
                            }

                            rose_response.clone().on_hover_cursor(CursorIcon::PointingHand);

                            if rose_response.clicked() {
                                let result = self.image_handler.select_image(&self.monitor_size);

                                match result {
                                    Ok(_) => {
                                        self.image_handler.load_image(
                                            self.config.image.loading.gui.lazy_loading,
                                            false,
                                            &mut self.notifier,
                                            &self.monitor_size,
                                            self.config.misc.experimental.use_fast_roseate_backend
                                        );
                                    },
                                    Err(error) => {
                                        self.notifier.toasts.lock().unwrap().toast_and_log(error.into(), ToastLevel::Error)
                                            .duration(Some(Duration::from_secs(5)));
                                    },
                                }
                            }
                        }
                    );

                    if file_is_hovering {
                        let rect = rose_rect.expand2(
                            Vec2::new(150.0, 100.0)
                        );
                        let painter = ui.painter();

                        let top_right = rect.right_top();
                        let top_left = rect.left_top();
                        let bottom_right = rect.right_bottom();
                        let bottom_left = rect.left_bottom();

                        self.draw_dotted_line(painter, &[top_left, top_right]);
                        self.draw_dotted_line(painter, &[top_right, bottom_right]);
                        self.draw_dotted_line(painter, &[bottom_right, bottom_left]);
                        self.draw_dotted_line(painter, &[bottom_left, top_left]);
                    }
                });

                return; // don't do anything else, you know, like stop right there bitch
            }

            self.info_box.update(ctx);
            self.zoom_pan.update(ctx);
            self.image_handler.update(
                &self.zoom_pan,
                &self.monitor_size,
                &mut self.notifier,
                self.config.misc.experimental.use_fast_roseate_backend
            );
            self.magnification_panel.update(ctx, &mut self.zoom_pan);

            let image = self.image_handler.image.clone().unwrap();

            if self.image_handler.image_loaded {
                ui.centered_and_justified(|ui| {
                    let scaled_image_size = self.window_scaling.relative_image_size(
                        Vec2::new(image.image_size.width as f32, image.image_size.height as f32)
                    );

                    // TODO: umm I think we should move this to self.zoom_pan.update() 
                    // and then move that function in here as we need `scaled_image_size`.
                    if self.zoom_pan.is_pan_out_of_bounds(scaled_image_size) {
                        self.zoom_pan.schedule_pan_reset(Duration::from_millis(300));

                        // As resetting the pan will just snap us back to the center 
                        // of the image we might as well schedule a reset for image scale too.
                        self.zoom_pan.schedule_scale_reset(Duration::from_millis(300));
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

                    self.zoom_pan.handle_pan_input(ctx, &response);
                    self.zoom_pan.handle_zoom_input(ctx, &response);
                });

                // We must update the WindowScaling with the window size AFTER
                // the image has loaded to maintain that smooth scaling animation on image show.
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
                if let Ok(loading_status) = self.notifier.loading_status.try_read() {
                    if let Some(loading) = loading_status.as_ref() {
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
            }
        );

    }
}