use core::f32;

use cirrus_theming::v1::Theme;
use cirrus_egui::v1::{config_manager::ConfigManager, notifier::Notifier};
use egui::{Color32, Context, CornerRadius, CursorIcon, Frame, Margin, Pos2, Rect, Sense, Vec2};
use zune_image::codecs::jpeg_xl::jxl_oxide::bitstream::BundleDefault;

use crate::{config::config::Config, image_handler::{ImageHandler}, magnification_panel::MagnificationPanel, monitor_size::MonitorSize};

pub struct Roseate {
    theme: Theme,
    notifier: Notifier,
    config_manager: ConfigManager<Config>,

    image_handler: ImageHandler,
    monitor_size: MonitorSize,
    magnification_panel: MagnificationPanel,

    zoom: f32,
    offset: Vec2,
    last_drag: Option<Pos2>,
}

impl Roseate {
    pub fn new(
        mut image_handler: ImageHandler,
        monitor_size: MonitorSize,
        theme: Theme,
        mut notifier: Notifier,
        config_manager: ConfigManager<Config>
    ) -> Self {
        let config = &config_manager.config;

        if image_handler.image.is_some() {
            image_handler.load_image(
                config.image.loading.initial.lazy_loading,
                &mut notifier,
                &monitor_size,
                config.misc.experimental.get_image_processing_backend()
            );
        }

        let magnification_panel = MagnificationPanel::new(config, &mut notifier);

        Self {
            theme,
            notifier,
            image_handler,
            monitor_size,
            magnification_panel,
            config_manager,
            zoom: 1.0,
            offset: Vec2::ZERO,
            last_drag: None,
        }
    }
}

impl eframe::App for Roseate {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.magnification_panel.handle_input(ctx);

        let central_panel_frame = Frame {
            inner_margin: Margin::ZERO,
            outer_margin: Margin::ZERO,
            fill: Color32::from_hex(&self.theme.primary_colour.hex_code).unwrap(),
            ..Frame::default_with_context(ctx)
        };

        egui::CentralPanel::default()
            .frame(central_panel_frame)
            .show(ctx, |ui| {
            let config = &self.config_manager.config;

            self.notifier.update(ctx);
            self.image_handler.update(
                &ctx,
                &self.zoom,
                self.last_drag.is_some(),
                &self.monitor_size,
                &mut self.notifier,
                config.misc.experimental.get_image_processing_backend()
            );

            // NOTE: hopefully cloning this here doesn't duplicate anything big, I recall it shouldn't in my codebase.
            match (&self.image_handler.image.clone(), self.image_handler.image_loaded) {
                // TODO: in the future we'll have some sort of value 
                // that tells use that the image exists and is loading.
                (Some(image), true) => {
                    egui::Frame::NONE
                        .show(ui, |ui| {
                            let available_rect = ui.available_rect_before_wrap();

                            let response = ui.interact(
                                available_rect,
                                ui.id().with("image_viewport"),
                                Sense::click_and_drag()
                            );

                            let image_size = Vec2::new(
                                image.image_size.0 as f32, image.image_size.1 as f32
                            );

                            let image_size_relative_to_zoom = image_size * self.zoom;

                            // Center the image in the center plus the offset for panning.
                            // The "image_rect" controls entirely how the image should be painted in size and position.
                            let image_rect = Rect::from_center_size(
                                available_rect.center() + self.offset,
                                image_size_relative_to_zoom,
                            );

                            // Handle zoom
                            if response.hovered() {
                                let scroll = ui.input(|i| i.smooth_scroll_delta.y);

                                if scroll.abs() > 0.0 {
                                    // Mouse position relative to screen coordinates.
                                    let mouse_position = ui.input(|i| i.pointer.hover_pos())
                                        .unwrap_or(available_rect.center());

                                    let before_zoom = self.zoom;

                                    // TODO: configurable zoom speed (default is "0.005").
                                    let zoom_delta = (scroll * 0.005).exp(); // ".exp()" applies a smooth exponential zoom
                                    // TODO: configurable zoom factor limits, sensible values are currently in place but 
                                    // it would be FUNNY to zoom out of the entire galaxy and zoom in until maximum 32 bit 
                                    // unsigned floating point integer is reached (this is how it used to be before v1.0 alpha 17).
                                    self.zoom = (self.zoom * zoom_delta).clamp(0.01, 100.0);

                                    // Zoom into mouse cursor using offset.
                                    let before_relative_mouse_position = (mouse_position - image_rect.center()) / before_zoom;
                                    let relative_mouse_position = (mouse_position - image_rect.center()) / self.zoom;

                                    self.offset += (relative_mouse_position - before_relative_mouse_position) * before_zoom;
                                }
                            }

                            // Handle panning
                            if response.dragged() {
                                let delta = response.drag_delta();
                                self.offset += delta;

                                // I kinda like the grabbing cursor.
                                ui.ctx().set_cursor_icon(CursorIcon::Grabbing);

                                // ui.ctx().request_repaint();
                            }

                            let egui_image = self.image_handler.get_egui_image(ctx)
                                .corner_radius(10.0);

                            // Drawing the image to the viewport
                            egui_image.paint_at(ui, image_rect);
                        });

                    ctx.request_repaint_after_secs(0.5); // We need to request repaints just in 
                    // just in case one doesn't happen when the window is resized in a certain circumstance 
                    // (i.e. the user maximizes the window and doesn't interact with it). I'm not sure how else we can fix it.
                },
                _ => {

                },
            }
        });

        // This is deliberately placed after the central panel so the central panel 
        // can take up all the space essentially ignoring the space this panel would otherwise take.
        // Check out the egui docs for more clarification: https://docs.rs/egui/0.32.3/egui/containers/panel/struct.CentralPanel.html
        egui::TopBottomPanel::bottom("status_bar")
            .frame(Frame::NONE)
            .show_separator_line(false)
            .show(ctx, |ui| {
                if let Some(loading) = &self.notifier.loading {
                    Frame::default()
                        .fill(Color32::from_hex(&self.theme.primary_colour.hex_code).unwrap())
                        .inner_margin(Margin::symmetric(10, 6))
                        .corner_radius(CornerRadius {ne: 10, ..Default::default()})
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.add(
                                    egui::Spinner::new()
                                        .color(Color32::from_hex("#e05f78").unwrap()) // NOTE: This should be the default accent colour.
                                        .size(20.0)
                                );

                                if let Some(message) = &loading.message {
                                    ui.label(message);
                                }
                            });
                        });
                }
            }
        );
    }
}