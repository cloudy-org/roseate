#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::{path::Path, time::Duration};

use cirrus_egui::v1::{config_manager::{ConfigManager}, notifier::Notifier, styling::Styling};
use config::config::Config;
use env_logger::Env;
use image_handler::{ImageHandler};
use log::debug;
use eframe::egui;
use egui_notify::ToastLevel;
use cirrus_theming::v1::{Colour, Theme};
use clap::{arg, command, Parser};

use error::Error;
use app::Roseate;
use monitor_size::MonitorSize;

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

static APP_NAME: &str = "roseate";
static TEMPLATE_CONFIG_TOML_STRING: &str = include_str!("../assets/config.template.toml");

/// 🌹 A fast as fuck, memory efficient and simple but fancy image viewer built with 🦀 Rust that's cross platform.
#[derive(Parser, Debug)]
#[clap(author = "Goldy")]
#[command(version, about, long_about = None)]
struct Args {
    /// Valid path to image.
    image: Option<String>,

    /// Valid themes at the moment: dark, light
    #[arg(short, long)]
    theme: Option<String>,
}

fn main() -> eframe::Result {
    let logger_env = Env::default()
        .filter_or("RUST_LOG", "warn");

    env_logger::init_from_env(logger_env);

    // Modern GUI image viewers should never silently 
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
        match &config.misc.override_monitor_size {
            Some(size) => Some((size.width as f32, size.height as f32)),
            None => None,
        }
    );

    monitor_size.fetch_from_cache();

    if !monitor_size.exists() {
        // we should be 100% safe to unwrap here 
        // as we're the first ones to access notifier.toasts at this point.
        notifier.toast(
            "The monitor size was not cached yet so the \
            image MAY appear a little blurry or over sharpened at first. Roseate will \
            clear this up and this should never happen again the next time you launch Roseate.",
            ToastLevel::Warning,
            |toast| {
                toast.duration(Some(Duration::from_secs(10)));
            }
        )
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
    let theme_string = cli_args.theme;

    let mut image_handler = ImageHandler::new();

    if let Some(path) = image_path {
        debug!("Image '{}' loading from path...", path);

        let path = Path::new(&path);

        if !path.exists() {
            let error = Error::FileNotFound(
                None,
                path.to_path_buf(),
                "That file doesn't exist!".to_string()
            );

            notifier.toast(
                Box::new(error),
                ToastLevel::Error,
                |toast| {
                    toast.duration(Some(Duration::from_secs(10)));
                }
            )
        } else {
            let configured_image_optimizations = config.image.optimizations.get_optimizations();

            let result = image_handler.init_image(path, configured_image_optimizations);

            if let Err(error) = result {
                notifier.toast(
                    Box::new(error),
                    ToastLevel::Error,
                    |_| {}
                );
            }
        }
    }

    let is_dark = match theme_string {
        Some(string) => {
            if string == "light" {
                false
            } else if string == "dark" {
                true
            } else {
                log::warn!(
                    "'{}' is not a valid theme. Pass either 'dark' or 'light'.", string
                );

                true
            }
        },
        _ => true
    };

    let theme_colours = match is_dark {
        true => vec![
            Colour::from_hex("#0a0909"),
            Colour::from_hex("#201f1f"),
            Colour::from_hex("#494848"),
        ],
        false => vec![
            Colour::from_hex("#b4dede"),
            Colour::from_hex("#aec5d4"),
            Colour::from_hex("#57575b"),
        ],
    };

    let theme = Theme::new(
        is_dark,
        theme_colours,
        Some(
            Colour::from_hex("#e05f78")
        )
    );

    eframe::run_native(
        "Roseate",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Styling::new(&theme, None)
                .set_all()
                .apply(&cc.egui_ctx);

            if image_handler.image.is_some() {
                image_handler.load_image(
                    &cc.egui_ctx,
                    config_manager.config.image.loading.initial.lazy_loading,
                    &mut notifier,
                    &monitor_size,
                    config_manager.config.misc.experimental.get_image_processing_backend()
                );
            }

            let app = Roseate::new(image_handler, monitor_size, theme, notifier, config_manager);

            Ok(Box::new(app))
        }),
    )
}