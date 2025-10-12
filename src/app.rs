use core::f32;

use cirrus_theming::v1::Theme;
use cirrus_egui::v1::{config_manager::ConfigManager, notifier::Notifier};
use egui::{Color32, Context, Event, Frame, Margin, Modifiers, Rect, Response, Sense, UiBuilder, Vec2};
use zune_image::codecs::jpeg_xl::jxl_oxide::bitstream::BundleDefault;

use crate::{config::config::Config, image_handler::{self, ImageHandler}, magnification_panel::MagnificationPanel, monitor_size::MonitorSize, zoom_pan::ZoomPan};

pub struct Roseate {
    theme: Theme,
    notifier: Notifier,
    config_manager: ConfigManager<Config>,

    image_handler: ImageHandler,
    monitor_size: MonitorSize,
    magnification_panel: MagnificationPanel,

    image_scene_rect: Rect,
    image_scene_initial_rect: Option<Rect>,
    image_scene_zoom_factor: f32,
    image_scene_is_panning: bool,
    image_scene_response: Option<Response>
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
            image_scene_rect: Rect::ZERO,
            image_scene_initial_rect: None,
            image_scene_zoom_factor: 1.0,
            image_scene_is_panning: false,
            image_scene_response: None
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
                &self.image_scene_zoom_factor,
                self.image_scene_is_panning,
                &self.monitor_size,
                &mut self.notifier,
                config.misc.experimental.get_image_processing_backend()
            );

            // NOTE: hopefully cloning this here doesn't duplicate anything big, I recall it shouldn't in my codebase.
            if let Some(image) = &self.image_handler.image.clone() {
                let image_size = Vec2::new(
                    image.image_size.0 as f32, image.image_size.1 as f32
                );

                // we're getting the available rect (space) so we can then use 
                // that to move the image's scene away from the top-left position to 
                // center of the window (effectively centering the image).
                let available_rect = ui.available_rect_before_wrap();

                // Move image's scene top-left position to center.
                let center = available_rect.min + (available_rect.size() - image_size) * 0.5;

                // Now we need to create a rect based off the center position 
                // for the new isolated UI that we'll be drawing the scene inside.
                let max_rect = Rect::from_min_size(center, image_size);

                let mut isolated_ui = ui.new_child(
                    UiBuilder::default()
                        .max_rect(max_rect)
                        .layout(*ui.layout())
                );

                let image_scene_response = egui::Scene::default()
                    .zoom_range(0.001..=f32::MAX)
                    .show(&mut isolated_ui, &mut self.image_scene_rect, |ui| {
                            let egui_image = self.image_handler.get_egui_image(ctx)
                                .corner_radius(10.0);

                            ui.add(egui_image);
                        }
                    ).response;

                let image_scene_initial_rect = self.image_scene_initial_rect.get_or_insert(
                    self.image_scene_rect
                );

                let initial_size = image_scene_initial_rect.size();
                let current_size = self.image_scene_rect.size();

                self.image_scene_zoom_factor = (
                    initial_size.x / current_size.x + initial_size.y / current_size.y
                ) * 0.5;

                self.image_scene_is_panning = image_scene_response.dragged();
                self.image_scene_response = Some(image_scene_response);

                // ctx.request_repaint_after_secs(0.5); // We need to request repaints just in 
                // // just in case one doesn't happen when the window is resized in a certain circumstance 
                // // (i.e. the user maximizes the window and doesn't interact with it). I'm not sure how else we can fix it.
            }
        });

    }

    fn raw_input_hook(&mut self, _ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        if let Some(image_scene_response) = &self.image_scene_response {
            // we don't want to modify mouse wheel event's modifiers 
            // unless the user is interacting with our image scene widget.
            if image_scene_response.contains_pointer() == false {
                return;
            }

            for event in &mut raw_input.events {
                if let Event::MouseWheel { modifiers, .. } = event {
                    // this is a temporary solution to enable zooming in / out in the
                    // egui::Scene widget without holding a CTRL key.
                    // 
                    // So here we are spoofing that modifier key press.
                    *modifiers = Modifiers::CTRL;
                }
            }
        }
    }
}