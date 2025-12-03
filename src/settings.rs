use cirrus_config::config_key_path;
use cirrus_egui::v1::widgets::settings::{Settings, section::{Section, SectionDisplayInfo, SectionOverrides}};
use cirrus_theming::v1::Theme;
use egui::Ui;

use crate::{TEMPLATE_CONFIG_TOML_STRING, config::config::Config};

pub struct SettingsMenu {}

impl SettingsMenu {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(&self, ui: &mut Ui, theme: &Theme, config: &mut Config) {
        Settings::new(TEMPLATE_CONFIG_TOML_STRING, &ui)
        // .add_section(
        //     Section::new(
        //         config_key_path!(config.image.optimizations.mode),
        //         &mut config.image.optimizations.mode,
        //         SectionOverrides {
        //             choices: Some([
        //                 Some("default"),
        //                 Some("speed"),
        //             ]),
        //             ..Default::default()
        //         },
        //         SectionDisplayInfo {
        //             name: Some("Image optimization mode".into()),
        //             ..Default::default()
        //         }
        //     )
        // )
        .add_section(
            Section::new(
                config_key_path!(config.ui.magnification_panel.enabled_default),
                &mut config.ui.magnification_panel.enabled_default,
                SectionOverrides::default(),
                SectionDisplayInfo {
                    name: Some("Display Magnification Panel".into()),
                    ..Default::default()
                }
            )
        )
        .add_section(
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
        )
        .add_section(
            Section::new(
                config_key_path!(config.ui.viewport.zoom_into_cursor),
                &mut config.ui.viewport.zoom_into_cursor,
                SectionOverrides::default(),
                SectionDisplayInfo::default()
            )
        )
        .add_section(
            Section::new(
                config_key_path!(config.ui.viewport.fit_to_window),
                &mut config.ui.viewport.fit_to_window,
                SectionOverrides::default(),
                SectionDisplayInfo {
                    name: Some("Fit image to window".into()),
                    ..Default::default()
                }
            )
        )
        .add_section(
            Section::new(
                config_key_path!(config.ui.viewport.animate_fit_to_window),
                &mut config.ui.viewport.animate_fit_to_window,
                SectionOverrides::default(),
                SectionDisplayInfo {
                    name: Some("Animate fit image to window".into()),
                    ..Default::default()
                }
            )
        )
        .add_section(
            Section::new(
                config_key_path!(config.ui.viewport.animate_reset),
                &mut config.ui.viewport.animate_reset,
                SectionOverrides::default(),
                SectionDisplayInfo {
                    name: Some("Animate viewport reset".into()),
                    ..Default::default()
                }
            )
        )
        .add_section(
            Section::new(
                config_key_path!(config.image.loading.initial.lazy_loading),
                &mut config.image.loading.initial.lazy_loading,
                SectionOverrides::default(),
                SectionDisplayInfo {
                    name: Some("Lazy load image initially".into()),
                    ..Default::default()
                }
            )
        )
        .add_section(
            Section::new(
                config_key_path!(config.image.loading.gui.lazy_loading),
                &mut config.image.loading.gui.lazy_loading,
                SectionOverrides::default(),
                SectionDisplayInfo {
                    name: Some("Lazy load Image in GUI".into()),
                    ..Default::default()
                }
            )
        ).show_ui(ui, &theme);
    }
}