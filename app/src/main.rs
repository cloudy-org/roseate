#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::{path::Path, time::Duration};

use cirrus_egui::v1::{config_manager::{ConfigManager}, notifier::Notifier, styling::Styling};
use cirrus_theming::v1::{colour::Colour, theme::Theme};
use config::config::Config;
use env_logger::Builder;
use image_handler::{ImageHandler};
use log::{LevelFilter, info};
use eframe::egui;
use egui_notify::ToastLevel;
use clap::{command, Parser};

use app::Roseate;
use monitor_size::MonitorSize;

use crate::image::image::Image;

mod app;
mod utils;
mod files;
mod image;
mod error;
mod config;
mod windows;
mod image_handler;
mod ui_controls;
mod about_window;
mod image_selection_menu;
mod monitor_size;
mod viewport;
mod settings;
mod context_menu;
mod tutorial;

static APP_NAME: &str = "roseate";
static TEMPLATE_CONFIG_TOML_STRING: &str = include_str!("../assets/config.template.toml");

/// 🌹 A fast as fuck, memory efficient and simple but fancy image viewer built with 🦀 Rust that's cross platform.
#[derive(Parser, Debug)]
#[clap(author = "Goldy")]
#[command(version, about, long_about = None)]
struct Args {
    /// Valid path to image.
    image: Option<String>,
}

fn main() -> eframe::Result {
    Builder::from_default_env()
        .filter_level(LevelFilter::Warn)
        .filter_module("zbus", LevelFilter::Off)
        .filter_module("sctk", LevelFilter::Off)
        .filter_module("winit", LevelFilter::Off)
        .filter_module("tracing", LevelFilter::Off)
        .parse_default_env()
        .init();

    // Modern GUI applications should never silently
    // error and exit without visually notifying the user
    // hence I have brought toasts outside the scope of app::Roseate
    // so we can queue up notifications when things go wrong here.
    let mut notifier = Notifier::new();

    let config_manager: ConfigManager<Config> = match ConfigManager::new(APP_NAME, TEMPLATE_CONFIG_TOML_STRING) {
        Ok(config_manager) => config_manager,
        Err(error) => {
            notifier.toast(
                format!(
                    "Error occurred initializing roseate's config file! \
                    Falling back to default config! Error: {}", error.human_message()
                ),
                ToastLevel::Error,
                |toast| {
                    toast.duration(Some(Duration::from_secs(10)));
                }
            );

            ConfigManager::default()
        }
    };

    let config = &config_manager.config;

    // TODO: fill monitor size params with values from config
    let mut monitor_size = MonitorSize::new(
        None,
        match &config_manager.config.misc.override_monitor_size {
            Some(size) => Some((size.width as f32, size.height as f32)),
            None => None,
        }
    );

    monitor_size.fetch_from_cache();

    if !monitor_size.exists() {
        notifier.toast(
            "The monitor size was not cached yet so the \
            image MAY appear a little blurry or over sharpened at first. Roseate will \
            clear this up and this should never happen again the next time you launch Roseate.",
            ToastLevel::Warning,
            |toast| {
                toast.duration(Some(Duration::from_secs(10)));
            }
        );
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_drag_and_drop(true),
        multisampling: 0,
        vsync: true,
        ..Default::default()
    };

    let cli_args = Args::parse();

    let image_path = cli_args.image;

    let image_optimizations = config.image.optimizations.get_optimizations()
        .normalize();

    let mut image_handler = match image_path {
        Some(path) => {
            info!("Image '{}' loading from path...", path);

            let path = Path::new(&path).to_owned();

            let image = match Image::new(path) {
                Ok(image) => Some(image),
                Err(error) => {
                    notifier.toast(
                        Box::new(error),
                        ToastLevel::Error,
                        |toast| {
                            toast.duration(Some(Duration::from_secs(10)));
                        }
                    );

                    None
                },
            };

            ImageHandler::new(image, image_optimizations)
        },
        None => ImageHandler::new(None, image_optimizations),
    };

    image_handler.load_image(
        config.image.loading.initial.lazy_loading,
        config.image.backend.get_decoding_backend(),
        &monitor_size,
        &mut notifier,
    );

    let theme = Theme::new(Some(Colour::from_hex(0xe05f78)));

    eframe::run_native(
        "Roseate",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Styling::new(&theme)
                .set_all()
                .apply(&cc.egui_ctx);

            let app = Roseate::new(
                image_handler,
                monitor_size,
                theme,
                notifier,
                config_manager
            );

            Ok(Box::new(app))
        }),
    )
}
