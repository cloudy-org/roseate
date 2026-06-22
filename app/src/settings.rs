use cirrus_config::{config_key_path, template::Template};
use cirrus_egui::widgets::settings::{Settings, any_section::AnySection, section::{Section, SectionDisplayInfo, SectionOverrides}};
use cirrus_theming::theme::Theme;
use eframe::egui::Ui;

use crate::{TEMPLATE_CONFIG_TOML_STRING, config::config::Config};

pub struct SettingsMenu {
    // NOTE: Set to static as "TEMPLATE_CONFIG_TOML_STRING" is essentially static
    template_config: Template<'static>,
}

impl SettingsMenu {
    pub fn new() -> Self {
        // may move this to app.rs
        let mut template_config = Template::new(TEMPLATE_CONFIG_TOML_STRING);

        // TODO: when template config is moved, notify user when parsing fails.
        template_config.parse_keys().unwrap();

        Self {
            template_config
        }
    }

    pub fn show(&mut self, ui: &mut Ui, theme: &Theme, config: &mut Config) {
        let mut settings = Settings::new();

        // image optimization
        settings.add_section(
            Section::new(
                config_key_path!(config.image.optimizations.mode),
                &mut config.image.optimizations.mode,
                SectionOverrides {
                    choices: Some(
                        vec![
                            String::from("balanced").into(),
                            String::from("speed").into(),
                            String::from("quality").into(),
                        ]
                    ),
                    ..Default::default()
                },
                SectionDisplayInfo {
                    name: Some("Image optimization mode".into()),
                    ..Default::default()
                }
            )
        );

        // ui controls
        settings.add_section(
            AnySection::ChildSections {
                title: String::from("UI Controls"),
                sections: vec![
                    Section::new(
                        config_key_path!(config.ui.controls.show),
                        &mut config.ui.controls.show,
                        SectionOverrides::default(),
                        SectionDisplayInfo {
                            name: Some("Show all controls".into()),
                            ..Default::default()
                        }
                    ).into(),
                    Section::new(
                        config_key_path!(config.ui.controls.settings),
                        &mut config.ui.controls.settings,
                        SectionOverrides::default(),
                        SectionDisplayInfo {
                            name: Some("Show settings button".into()),
                            ..Default::default()
                        }
                    ).into(),
                    Section::new(
                        config_key_path!(config.ui.controls.fullscreen),
                        &mut config.ui.controls.fullscreen,
                        SectionOverrides::default(),
                        SectionDisplayInfo {
                            name: Some("Show fullscreen button".into()),
                            ..Default::default()
                        }
                    ).into(),
                    Section::new(
                        config_key_path!(config.ui.controls.magnification),
                        &mut config.ui.controls.magnification,
                        SectionOverrides::default(),
                        SectionDisplayInfo {
                            name: Some("Show magnification panel".into()),
                            ..Default::default()
                        }
                    ).into()
                ]
            }
        );

        // viewport
        settings.add_section(
            AnySection::ChildSections {
                title: String::from("Viewport"),
                sections: vec![
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
                    ).into(),
                    Section::new(
                        config_key_path!(config.ui.viewport.zoom_into_cursor),
                        &mut config.ui.viewport.zoom_into_cursor,
                        SectionOverrides::default(),
                        SectionDisplayInfo::default()
                    ).into(),
                    Section::new(
                        config_key_path!(config.ui.viewport.fit_to_window),
                        &mut config.ui.viewport.fit_to_window,
                        SectionOverrides::default(),
                        SectionDisplayInfo {
                            name: Some("Fit image to window".into()),
                            ..Default::default()
                        }
                    ).into(),
                    Section::new(
                        config_key_path!(config.ui.viewport.animate_fit_to_window),
                        &mut config.ui.viewport.animate_fit_to_window,
                        SectionOverrides::default(),
                        SectionDisplayInfo {
                            name: Some("Animate fit image to window".into()),
                            ..Default::default()
                        }
                    ).into(),
                    Section::new(
                        config_key_path!(config.ui.viewport.animate_reset),
                        &mut config.ui.viewport.animate_reset,
                        SectionOverrides::default(),
                        SectionDisplayInfo {
                            name: Some("Animate viewport reset".into()),
                            ..Default::default()
                        }
                    ).into()
                ]
            }
        );

        settings.add_section(
            AnySection::ChildSections {
                title: String::from("Home Menu"),
                sections: vec![
                    Section::new(
                        config_key_path!(config.ui.home_menu.show_settings_button),
                        &mut config.ui.home_menu.show_settings_button,
                        SectionOverrides::default(),
                        SectionDisplayInfo::default(),
                    ).into(),
                    Section::new(
                        config_key_path!(config.ui.home_menu.show_open_image_button),
                        &mut config.ui.home_menu.show_open_image_button,
                        SectionOverrides::default(),
                        SectionDisplayInfo::default(),
                    ).into()
                ]
            }
        );

        settings.add_section(
            Section::new(
                config_key_path!(config.ui.image_info.show_location),
                &mut config.ui.image_info.show_location,
                SectionOverrides::default(),
                SectionDisplayInfo {
                    name: Some(String::from("Show image location")),
                    ..Default::default()
                },
            )
        );

        settings.add_section(
            Section::new(
                config_key_path!(config.image.loading.initial.lazy_loading),
                &mut config.image.loading.initial.lazy_loading,
                SectionOverrides::default(),
                SectionDisplayInfo {
                    name: Some("Lazy load image initially".into()),
                    ..Default::default()
                }
            )
        );

        settings.add_section(
            Section::new(
                config_key_path!(config.image.backend.decoder),
                &mut config.image.backend.decoder,
                SectionOverrides {
                    choices: Some(
                        vec![
                            "image-rs".into()
                        ]
                    ),
                    ..Default::default()
                },
                SectionDisplayInfo::default()
            )
        );

        settings.show_ui(
            ui,
            &theme,
            &self.template_config.keys
        );
    }
}