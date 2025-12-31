use cirrus_egui::v1::{config_manager::ConfigManager, notifier::Notifier, widgets::settings::Settings};
use cirrus_theming::v1::theme::Theme;
use egui::{Color32, Context, CornerRadius, Frame, Key, Margin};

use crate::{about_window::AboutWindow, config::config::Config, context_menu::ContextMenu, image_handler::ImageHandler, image_selection_menu::ImageSelectionMenu, monitor_size::MonitorSize, settings::SettingsMenu, tutorial::Tutorial, ui_controls::UIControlsManager, viewport::Viewport, windows::WindowsManager};

pub struct Roseate {
    theme: Theme,
    notifier: Notifier,
    config_manager: ConfigManager<Config>,

    viewport: Viewport,
    image_handler: ImageHandler,
    monitor_size: MonitorSize,
    settings_menu: SettingsMenu,
    selection_menu: ImageSelectionMenu,
    about_window: AboutWindow<'static>,
    windows_manager: WindowsManager,
    ui_controls_manager: UIControlsManager,
    context_menu: ContextMenu,
    tutorial: Tutorial,

    show_settings: bool,
    show_about: bool,
}

impl Roseate {
    pub fn new(
        image_handler: ImageHandler,
        monitor_size: MonitorSize,
        theme: Theme,
        notifier: Notifier,
        config_manager: ConfigManager<Config>
    ) -> Self {
        let viewport = Viewport::new();
        let windows_manager = WindowsManager::new();
        let settings_menu = SettingsMenu::new();
        let about_window = AboutWindow::new();
        let selection_menu = ImageSelectionMenu::new();
        let ui_controls_manager = UIControlsManager::new();
        let context_menu = ContextMenu::new();
        let tutorial = Tutorial::new();

        Self {
            theme,
            notifier,
            viewport,
            image_handler,
            monitor_size,
            settings_menu,
            selection_menu,
            about_window,
            windows_manager,
            ui_controls_manager,
            config_manager,
            context_menu,
            tutorial,

            show_settings: false,
            show_about: false,
        }
    }
}

impl eframe::App for Roseate {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        Settings::handle_input(
            &ctx,
            &mut self.config_manager,
            &mut self.notifier,
            &mut self.show_settings
        );

        let config = &self.config_manager.config;

        self.windows_manager.handle_input(&ctx);
        self.ui_controls_manager.handle_input(&ctx, config.ui.controls.hide);

        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(Key::A)) {
            self.show_about = !self.show_about;
        }

        // In roseate I prefer the central panel with zero margin.
        let central_panel_frame = Frame::default()
            .inner_margin(Margin::ZERO)
            .outer_margin(Margin::ZERO)
            .fill(Color32::from_hex(&self.theme.pallet.primary.to_hex_string()).unwrap());

        egui::CentralPanel::default()
            .frame(central_panel_frame)
            .show(ctx, |ui| {
            let config = &self.config_manager.config.clone();

            self.notifier.update(ctx);
            self.image_handler.update(
                &ctx,
                &self.viewport.zoom,
                self.viewport.is_busy,
                &self.monitor_size,
                config.image.backend.get_decoding_backend(),
                &mut self.notifier,
            );

            self.tutorial.show(ui, &mut self.config_manager);

            if self.show_settings {
                // we only want to run the config manager's
                // update loop when were are in the settings menu
                self.config_manager.update(ctx, &mut self.notifier);

                self.settings_menu.show(
                    ui,
                    &self.theme,
                    &mut self.config_manager.config
                );

                return;
            }

            if self.show_about {
                self.about_window.show(ui);
            }

            // NOTE: hopefully cloning this here doesn't duplicate anything big, I recall it shouldn't in my codebase.
            match (self.image_handler.image.as_ref(), self.image_handler.resource.as_ref()) {
                // TODO: in the future we'll have some sort of value
                // that tells use that the image exists and is loading.
                (Some(image), Some(image_resource))=> {
                    egui::Frame::NONE
                        .show(ui, |ui| {
                            // handle inputs here that you do not 
                            // want toggling outside the viewport
                            self.context_menu.handle_input(&ctx, &self.windows_manager);

                            self.windows_manager.show(
                                ui,
                                image_resource,
                                &self.image_handler.image_optimizations,
                                image,
                                // leaving this unwrap here for now, I'll defiantly improve this soon
                                self.image_handler.decoded_image_info.as_ref().unwrap()
                            );

                            self.context_menu.show(ui, &mut self.windows_manager);
                            self.ui_controls_manager.show(ui, &mut self.viewport);

                            let config_padding = config.ui.viewport.padding;
                            let proper_padding_percentage = ((100.0 - config_padding) / 100.0).clamp(0.0, 1.0);

                            self.viewport.show(
                                ui,
                                &image.size,
                                image_resource.clone(), // ImageHandlerData is safe to clone
                                proper_padding_percentage,
                                config.ui.viewport.zoom_into_cursor,
                                config.ui.viewport.fit_to_window,
                                config.ui.viewport.animate_fit_to_window,
                                config.ui.viewport.animate_reset
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
                                &mut self.notifier,
                                &self.monitor_size,
                                config.image.backend.get_decoding_backend(),
                                &self.theme.pallet.accent,
                                config.ui.selection_menu.show_open_image_button,
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
                        .fill(Color32::from_hex(&self.theme.pallet.primary.to_hex_string()).unwrap())
                        .inner_margin(Margin::same(8))
                        .corner_radius(CornerRadius {ne: 10, ..Default::default()})
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.add(
                                    egui::Spinner::new()
                                        .color(Color32::from_hex(&self.theme.pallet.accent.to_hex_string()).unwrap())
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
