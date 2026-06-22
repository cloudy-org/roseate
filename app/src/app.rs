use std::time::Duration;

use cirrus_authors::Authors;
use cirrus_egui::{config_manager::ConfigManager, notifier::{Notifier, banner::{BannerPlacement, BannerText}}, widgets::settings::Settings};
use cirrus_soft_binds::egui::{BoxedEguiInputReaderFunc, parse_and_get_egui_input_reader_from_string};
use cirrus_theming::theme::Theme;
use eframe::egui::{self, Color32, Context, CornerRadius, Frame, Key, Margin, ViewportCommand};
use egui_notify::ToastLevel;

use crate::{about_window::AboutWindow, config::config::Config, context_menu::ContextMenu, home_menu::HomeMenu, image_loader::ImageLoader, image_selector::ImageSelector, monitor_size::MonitorSize, settings::SettingsMenu, tutorial::Tutorial, ui_controls::UIControlsManager, viewport::Viewport, windows::WindowsManager};

pub struct Roseate {
    theme: Theme,
    notifier: Notifier,
    config_manager: ConfigManager<Config>,
    authors: Authors,

    viewport: Viewport,
    about_window: AboutWindow,
    image_loader: ImageLoader,
    image_selector: ImageSelector,
    monitor_size: MonitorSize,
    settings_menu: SettingsMenu,
    home_menu: HomeMenu,
    windows_manager: WindowsManager,
    ui_controls_manager: UIControlsManager,
    context_menu: ContextMenu,
    tutorial: Tutorial,

    open_image_input_reader: Option<BoxedEguiInputReaderFunc>,

    show_settings: bool,
    show_about: bool,
    show_license: bool,
}

impl Roseate {
    pub fn new(
        image_selector: ImageSelector,
        image_loader: ImageLoader,
        monitor_size: MonitorSize,
        theme: Theme,
        notifier: Notifier,
        config_manager: ConfigManager<Config>,
        authors: Authors,
    ) -> Self {
        let viewport = Viewport::new();
        let about_window = AboutWindow::new();
        let windows_manager = WindowsManager::new();
        let settings_menu = SettingsMenu::new();
        let home_menu = HomeMenu::new();
        let ui_controls_manager = UIControlsManager::new();
        let context_menu = ContextMenu::new();
        let tutorial = Tutorial::new();

        Self {
            theme,
            notifier,
            authors,

            viewport,
            about_window,
            image_selector,
            image_loader,
            monitor_size,
            settings_menu,
            home_menu,
            windows_manager,
            ui_controls_manager,
            config_manager,
            context_menu,
            tutorial,

            open_image_input_reader: None,

            show_settings: false,
            show_about: false,
            show_license: false,
        }
    }

    fn handle_inputs(&mut self, ctx: &Context) {
        let config = &self.config_manager.config;

        self.windows_manager.handle_input(
            &ctx,
            &mut self.notifier,
            &config.key_binds.show_image_info,
            &config.key_binds.show_extra_image_info
        );
        self.ui_controls_manager.handle_input(
            &ctx,
            &mut self.notifier,
            &config.key_binds.show_ui_controls,
            config.ui.controls.show
        );

        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(Key::A)) {
            self.show_about = !self.show_about;
        }

        // toggle and escape fullscreen
        let is_fullscreen = ctx.input(
            |i| i.viewport().fullscreen.unwrap_or_default()
        );

        if is_fullscreen && ctx.input(|i| i.key_pressed(Key::Escape)) {
            ctx.send_viewport_cmd(
                ViewportCommand::Fullscreen(false)
            );

            self.notifier.show_banner(
                "Windowed Mode (ESC)",
                BannerPlacement::BOTTOM,
                Duration::from_secs(3)
            );
        }

        if ctx.input(|i| i.key_pressed(Key::F) || i.key_pressed(Key::F11)) {
            ctx.send_viewport_cmd(
                ViewportCommand::Fullscreen(!is_fullscreen)
            );

            self.notifier.show_banner(
                BannerText::new(
                    match is_fullscreen {
                        false => String::from("Fullscreen Mode (F11)"),
                        true => String::from("Windowed Mode")
                    },
                    match is_fullscreen {
                        false => Some(
                            String::from("Press 'F' / 'F11' again or 'ESCAPE' to exit.")
                        ),
                        true => None
                    }
                ),
                BannerPlacement::BOTTOM,
                Duration::from_secs(4)
            );
        }
    } 
}

impl eframe::App for Roseate {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // In roseate I prefer the central panel with zero margin.
        let central_panel_frame = Frame::default()
            .inner_margin(Margin::ZERO)
            .outer_margin(Margin::ZERO)
            .fill(Color32::from_hex(&self.theme.palette.primary.to_hex_string()).unwrap());

        egui::CentralPanel::default()
            .frame(central_panel_frame)
            .show(ctx, |ui| {
                // handle inputs and settings menu
                Settings::handle_input(
                    &ctx,
                    &mut self.config_manager,
                    &mut self.notifier,
                    &mut self.show_settings
                );

                // we have to render the settings menu here before inputs so typing into 
                // input boxes in the settings menu don't collide with configured key binds. 
                // A better way of handling this would be restructuring the entirety of the update 
                // loop to bring in a concept of "sections" where we have the home menu, the image 
                // viewport section and the settings menu as an individual section that is rendered 
                // on it's own so that we can choose to only listen to keybinds in one section but not 
                // the other.
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

                self.handle_inputs(&ctx);

                let config = &self.config_manager.config;

                let open_image_input_reader = self.open_image_input_reader.get_or_insert_with(
                    || {
                        match parse_and_get_egui_input_reader_from_string(
                            &config.key_binds.open_image,
                            |i, key| i.key_pressed(key)
                        ) {
                            Ok(reader) => Box::new(reader),
                            Err(error) => {
                                self.notifier.toast(
                                    error.to_string(),
                                    ToastLevel::Error,
                                    |_| {}
                                );

                                Box::new(
                                    |i| i.modifiers.ctrl && i.key_pressed(Key::O)
                                )
                            },
                        }
                    }
                );

                self.notifier.show(ui);
                // self.tutorial.show(ui, &mut self.config_manager);

                if self.show_about {
                    self.about_window.show(
                        ui,
                        &self.authors,
                        &mut self.show_license
                    );
                }

                self.image_loader.handle_input(
                    ui,
                    &mut self.image_selector,
                    &self.monitor_size,
                    config.image.backend.get_decoding_backend(),
                    &mut self.notifier,

                    open_image_input_reader
                );

                self.image_loader.dynamic_sampling_update(
                    &self.viewport.zoom,
                    self.viewport.is_busy,
                    &mut self.image_selector,
                    &self.monitor_size,
                    config.image.backend.clone().get_decoding_backend(),
                    &mut self.notifier,
                );

                let image_optimizations = self.image_loader.image_optimizations.clone();

                // TODO: should we pass optimizations into .upload() and hold them in app.rs??
                match self.image_loader.upload(ctx, &self.image_selector, &mut self.notifier) {
                    Some(uploaded_image) => {
                        egui::Frame::NONE
                            .show(ui, |ui| {
                                // handle inputs here that you do not
                                // want toggling outside the viewport
                                self.context_menu.handle_input(&ctx, &self.windows_manager);

                                self.windows_manager.show(
                                    ui,
                                    &uploaded_image,
                                    &image_optimizations,
                                    &self.monitor_size,
                                    config.ui.image_info.show_location,
                                );

                                self.context_menu.show(ui, &mut self.windows_manager);
                                self.ui_controls_manager.show(
                                    ui,
                                    &mut self.viewport,
                                    config.ui.controls.magnification,
                                    config.ui.controls.fullscreen,
                                    config.ui.controls.settings,
                                    &mut self.show_settings,
                                );

                                let config_padding = config.ui.viewport.padding;
                                let proper_padding_percentage = ((100.0 - config_padding) / 100.0).clamp(0.0, 1.0);

                                self.viewport.show(
                                    ui,
                                    &uploaded_image.image.size,
                                    uploaded_image.resource.clone(), // ImageResource is safe to clone without expensive dup
                                    &mut self.notifier,
                                    proper_padding_percentage,
                                    config.ui.viewport.zoom_into_cursor,
                                    config.ui.viewport.fit_to_window,
                                    config.ui.viewport.animate_fit_to_window,
                                    config.ui.viewport.animate_reset,
                                    &config.key_binds.reset_viewport
                                );
                            });

                        ctx.request_repaint_after_secs(0.5); // We need to request repaints just in
                        // just in case one doesn't happen when the window is resized in a certain circumstance
                        // (i.e. the user maximizes the window and doesn't interact with it). I'm not sure how else we can fix it.
                    },
                    None => {
                        egui::Frame::NONE
                            .show(ui, |ui| {
                                self.home_menu.show(
                                    ui,
                                    &mut self.image_selector,
                                    &mut self.image_loader,
                                    &mut self.notifier,
                                    &self.monitor_size,
                                    config.image.backend.get_decoding_backend(),
                                    &self.theme.palette.accent,

                                    &mut self.show_settings,

                                    config.ui.home_menu.show_open_image_button,
                                    config.ui.home_menu.show_settings_button,
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
                            .fill(Color32::from_hex(&self.theme.palette.primary.to_hex_string()).unwrap())
                            .inner_margin(Margin::same(8))
                            .corner_radius(CornerRadius {ne: 10, ..Default::default()})
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.add(
                                        egui::Spinner::new()
                                            .color(Color32::from_hex(&self.theme.palette.accent.to_hex_string()).unwrap())
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

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        log::info!("Cleaning up before exiting...");

        if let Err(error) = self.config_manager.save_if_changed() {
            log::error!(
                "Error occurred while saving config on exit! Error: {}",
                error.human_message()
            );
        }
    }
}
