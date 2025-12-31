use cirrus_config::config_key_path;
use cirrus_egui::v1::widgets::settings::{Settings, section::{Section, SectionDisplayInfo, SectionOverrides}};
use cirrus_theming::v1::theme::Theme;
use egui::Ui;

use crate::{TEMPLATE_CONFIG_TOML_STRING, config::config::Config};

pub struct SettingsMenu {}

impl SettingsMenu {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(&self, ui: &mut Ui, theme: &Theme, config: &mut Config) {
        let mut settings = Settings::new(TEMPLATE_CONFIG_TOML_STRING, &ui);

        let image_optimization_config_key_path = config_key_path!(config.image.optimizations.mode);

        if let Some(config_key) = &mut config.image.optimizations.mode {
            settings.add_section(
                Section::new(
                    image_optimization_config_key_path,
                    config_key,
                    SectionOverrides {
                        choices: Some([
                            "default".into(),
                            "speed".into(),
                        ].into()),
                        ..Default::default()
                    },
                    SectionDisplayInfo {
                        name: Some("Image optimization mode".into()),
                        ..Default::default()
                    }
                )
            );
        }

        settings.add_section(
            Section::new(
                config_key_path!(config.ui.controls.hide),
                &mut config.ui.controls.hide,
                SectionOverrides::default(),
                SectionDisplayInfo {
                    name: Some("Display Controls".into()),
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
                config_key_path!(config.ui.viewport.zoom_into_cursor),
                &mut config.ui.viewport.zoom_into_cursor,
                SectionOverrides::default(),
                SectionDisplayInfo::default()
            )
        ).add_section(
            Section::new(
                config_key_path!(config.ui.viewport.fit_to_window),
                &mut config.ui.viewport.fit_to_window,
                SectionOverrides::default(),
                SectionDisplayInfo {
                    name: Some("Fit image to window".into()),
                    ..Default::default()
                }
            )
        ).add_section(
            Section::new(
                config_key_path!(config.ui.viewport.animate_fit_to_window),
                &mut config.ui.viewport.animate_fit_to_window,
                SectionOverrides::default(),
                SectionDisplayInfo {
                    name: Some("Animate fit image to window".into()),
                    ..Default::default()
                }
            )
        ).add_section(
            Section::new(
                config_key_path!(config.ui.viewport.animate_reset),
                &mut config.ui.viewport.animate_reset,
                SectionOverrides::default(),
                SectionDisplayInfo {
                    name: Some("Animate viewport reset".into()),
                    ..Default::default()
                }
            )
        ).add_section(
            Section::new(
                config_key_path!(config.image.loading.initial.lazy_loading),
                &mut config.image.loading.initial.lazy_loading,
                SectionOverrides::default(),
                SectionDisplayInfo {
                    name: Some("Lazy load image initially".into()),
                    ..Default::default()
                }
            )
        ).add_section(
            Section::new(
                config_key_path!(config.image.loading.gui.lazy_loading),
                &mut config.image.loading.gui.lazy_loading,
                SectionOverrides::default(),
                SectionDisplayInfo {
                    name: Some("Lazy load Image in GUI".into()),
                    ..Default::default()
                }
            )
        );

        settings.show_ui(ui, &theme);
    }
}