use cirrus_theming::v1::Theme;
use cirrus_egui::v1::{config_manager::ConfigManager, notifier::Notifier};
use egui::{Color32, Context, CornerRadius, Frame, Margin};
use zune_image::codecs::jpeg_xl::jxl_oxide::bitstream::BundleDefault;

use crate::{config::config::Config, image_handler::ImageHandler, image_selection_menu::ImageSelectionMenu, magnification_panel::MagnificationPanel, monitor_size::MonitorSize, viewport::Viewport};

pub struct Roseate {
    theme: Theme,
    notifier: Notifier,
    config_manager: ConfigManager<Config>,

    viewport: Viewport,
    image_handler: ImageHandler,
    monitor_size: MonitorSize,
    selection_menu: ImageSelectionMenu,
    magnification_panel: MagnificationPanel,
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

        let viewport = Viewport::new();
        let selection_menu = ImageSelectionMenu::new();
        let magnification_panel = MagnificationPanel::new(config, &mut notifier);

        Self {
            theme,
            notifier,
            viewport,
            image_handler,
            monitor_size,
            selection_menu,
            magnification_panel,
            config_manager
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
                &self.viewport.zoom,
                false,
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
                            let egui_image = self.image_handler.get_egui_image(ctx);

                            let config_padding = config.ui.viewport.padding;
                            let proper_padding_percentage = ((100.0 - config_padding) / 100.0).clamp(0.0, 1.0);

                            self.viewport.show(
                                ui,
                                &image,
                                egui_image,
                                proper_padding_percentage,
                                config.ui.viewport.zoom_into_cursor,
                                true,
                                true,
                                true
                            );
                        });

                    ctx.request_repaint_after_secs(0.5); // We need to request repaints just in 
                    // just in case one doesn't happen when the window is resized in a certain circumstance 
                    // (i.e. the user maximizes the window and doesn't interact with it). I'm not sure how else we can fix it.
                },
                _ => {
                    egui::Frame::NONE
                        .show(ui, |ui| {
                            self.selection_menu.show(
                                ui,
                                &mut self.image_handler,
                                config.image.optimizations.get_optimizations(),
                                &mut self.notifier,
                                &self.monitor_size,
                                config.misc.experimental.get_image_processing_backend(),
                                &self.theme.accent_colour,
                                true // TODO: add to config
                            );
                        });
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