use std::time::Duration;

use cirrus_config::config_key_path;
use cirrus_egui::v1::{config_manager::ConfigManager, notifier::Notifier, widgets::settings::{section::{Section, SectionDisplayInfo, SectionOverrides}, Settings}};
use cirrus_theming::v1::Theme;
use eframe::egui::{self, Align, Color32, Context, CursorIcon, Frame, Layout, Margin, Rect, Stroke, Vec2};
use egui_notify::ToastLevel;

use crate::{config::config::Config, files, image_handler::{optimization::ImageOptimizations, ImageHandler}, magnification_panel::MagnificationPanel, monitor_size::MonitorSize, window_scaling::WindowScaling, windows::{about::AboutWindow, info::InfoWindow}, zoom_pan::ZoomPan, TEMPLATE_CONFIG_TOML_STRING};

pub struct Roseate<'a> {
    theme: Theme,
    zoom_pan: ZoomPan,
    info_box: InfoWindow,
    about_box: AboutWindow<'a>,
    notifier: Notifier,
    magnification_panel: MagnificationPanel,
    window_scaling: WindowScaling,
    last_window_rect: Rect,
    image_handler: ImageHandler,
    monitor_size: MonitorSize,
    config_manager: ConfigManager<Config>,
    show_settings: bool
}

impl<'a> Roseate<'a> {
    pub fn new(mut image_handler: ImageHandler, monitor_size: MonitorSize, mut notifier: Notifier, theme: Theme, config_manager: ConfigManager<Config>) -> Self {
        let config = &config_manager.config;

        if image_handler.image.is_some() {
            image_handler.load_image(
                config.image.loading.initial.lazy_loading,
                &mut notifier,
                &monitor_size,
                config.misc.experimental.get_image_processing_backend()
            );
        }

        let zoom_pan = ZoomPan::new(config, &mut notifier);
        let info_box = InfoWindow::new(config, &mut notifier);
        let about_box = AboutWindow::new(config, &mut notifier);
        let magnification_panel = MagnificationPanel::new(config, &mut notifier);

        Self {
            theme,
            notifier,
            zoom_pan,
            info_box,
            about_box,
            magnification_panel,
            window_scaling: WindowScaling::new(),
            last_window_rect: Rect::NOTHING,
            monitor_size,
            image_handler,
            config_manager,
            show_settings: false
        }
    }
}

impl eframe::App for Roseate<'_> {

    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.info_box.init(&self.image_handler.image);
        self.info_box.handle_input(ctx);

        self.zoom_pan.handle_reset_input(ctx);
        self.magnification_panel.handle_input(ctx);
        self.about_box.handle_input(ctx);

        Settings::handle_input(
            &ctx, &mut self.config_manager, &mut self.notifier, &mut self.show_settings
        );

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
            // the about box is to be toggle-able even without an image.

            if self.show_settings {
                // we only want to run the config manager's 
                // update loop when were are in the settings menu
                self.config_manager.update(ctx, &mut self.notifier);

                let config = &mut self.config_manager.config;

                Settings::new(TEMPLATE_CONFIG_TOML_STRING, &ui)
                    .add_section(
                        Section::new(
                            config_key_path!(config.ui.magnification_panel.enabled_default),
                            &mut config.ui.magnification_panel.enabled_default,
                            SectionOverrides::default(),
                            SectionDisplayInfo {
                                name: Some("Enable Magnification Panel".into()),
                                ..Default::default()
                            }
                        )
                    ).add_section(
                        Section::new(
                            config_key_path!(config.ui.viewport.padding),
                            &mut config.ui.viewport.padding,
                            SectionOverrides {
                                int_range: Some(0.0..=50.0),
                                ..Default::default()
                            },
                            SectionDisplayInfo {
                                name: Some("Viewport padding".into()),
                                ..Default::default()
                            }
                        )
                    ).add_section(
                        Section::new(
                            config_key_path!(config.image.loading.initial.lazy_loading),
                            &mut config.image.loading.initial.lazy_loading,
                            SectionOverrides::default(),
                            SectionDisplayInfo {
                                name: Some("Image initial lazy loading".into()),
                                ..Default::default()
                            }
                        )
                    ).add_section(
                        Section::new(
                            config_key_path!(config.image.loading.gui.lazy_loading),
                            &mut config.image.loading.gui.lazy_loading,
                            SectionOverrides::default(),
                            SectionDisplayInfo {
                                name: Some("Image GUI lazy loading".into()),
                                ..Default::default()
                            }
                        )
                    ).show_ui(ui, &self.theme);

                return;
            }

            let config = &self.config_manager.config;

            if self.image_handler.image.is_none() {
                let mut configured_image_optimizations = config.image.optimizations.get_optimizations();

                // TODO: remove this once we move DS to "[image.optimizations]".
                if config.misc.experimental.use_dynamic_sampling_optimization {
                    configured_image_optimizations.push(
                        ImageOptimizations::DynamicSampling(true, true)
                    );
                }

                // Collect dropped files.
                ctx.input(|i| {
                    let dropped_files = &i.raw.dropped_files;

                    if !dropped_files.is_empty() {
                        let path = dropped_files.first().unwrap()
                            .path
                            .as_ref()
                            .unwrap(); // gotta love rust ~ ananas

                        let result = self.image_handler.init_image(
                            path,
                            configured_image_optimizations.clone()
                        );

                        if let Err(error) = result {
                            self.notifier.toast(
                                Box::new(error),
                                ToastLevel::Error,
                                |_| {}
                            );
                            return;
                        }

                        self.image_handler.load_image(
                            true, 
                            &mut self.notifier,
                            &self.monitor_size,
                            config.misc.experimental.get_image_processing_backend()
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
                                let result = self.image_handler.select_image(
                                    configured_image_optimizations
                                );

                                match result {
                                    Ok(_) => {
                                        self.image_handler.load_image(
                                            config.image.loading.gui.lazy_loading,
                                            &mut self.notifier,
                                            &self.monitor_size,
                                            config.misc.experimental.get_image_processing_backend()
                                        );
                                    },
                                    Err(error) => {
                                        self.notifier.toast(
                                            Box::new(error),
                                            ToastLevel::Error,
                                            |toast| {
                                                toast.duration(Some(Duration::from_secs(5)));
                                            }
                                        );
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

                        // Draw dotted lines to indicate file being dropped.
                        for index in 0..4 {
                            let pos = match index {
                                0 => &[rect.left_top(), rect.right_top()],
                                1 => &[rect.right_top(), rect.right_bottom()],
                                2 => &[rect.right_bottom(), rect.left_bottom()],
                                3 => &[rect.left_bottom(), rect.left_top()],
                                _ => unreachable!()
                            };

                            painter.add(
                                egui::Shape::dashed_line(
                                    pos,
                                    Stroke {
                                        width: 2.0,
                                        color: Color32::from_hex(
                                            &self.theme.accent_colour.hex_code
                                        ).unwrap()
                                    },
                                    11.0,
                                    10.0
                                )
                            );
                        }
                    }
                });

                return; // don't do anything else, you know, like stop right there bitch
            }

            self.info_box.update(ctx);
            self.zoom_pan.update(ctx);
            self.image_handler.update(
                &ctx,
                &self.zoom_pan,
                &self.monitor_size,
                &mut self.notifier,
                config.misc.experimental.get_image_processing_backend()
            );
            self.magnification_panel.update(ctx, &mut self.zoom_pan);

            let image_size = self.image_handler.image.as_ref().unwrap().image_size;

            if self.image_handler.image_loaded {
                ui.centered_and_justified(|ui| {
                    let scaled_image_size = self.window_scaling.relative_image_size(
                        Vec2::new(image_size.0 as f32, image_size.1 as f32)
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

                    let egui_image = self.image_handler.get_egui_image(ctx);

                    egui_image
                        .rounding(10.0)
                        .paint_at(ui, zoom_pan_rect);

                    self.zoom_pan.handle_pan_input(ctx, &response);
                    self.zoom_pan.handle_zoom_input(ctx, &response);
                });

                // We must update the WindowScaling with the window size AFTER
                // the image has loaded to maintain that smooth scaling animation on image show.
                self.window_scaling.update(&window_rect, &image_size, config.ui.viewport.padding);

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
                if let Some(loading) = &self.notifier.loading {
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