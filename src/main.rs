#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::{env, path::Path, time::Duration};

use cirrus_egui::v1::styling::Styling;
use config::config::Config;
use image_handler::ImageHandler;
use log::debug;
use eframe::egui;
use egui_notify::ToastLevel;
use cirrus_theming::v1::Theme;
use clap::{arg, command, Parser};

use error::Error;
use app::Roseate;
use notifier::NotifierAPI;

mod app;
mod utils;
mod files;
mod image;
mod error;
mod config;
mod notifier;
mod windows;
mod zoom_pan;
mod image_handler;
mod window_scaling;
mod magnification_panel;

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
    if !env::var("RUST_LOG").is_ok() {
        env::set_var("RUST_LOG", "WARN");
    }

    env_logger::init();

    // Modern GUI image viewers should never silently 
    // error and exit without visually notifying the user 
    // hence I have brought toasts outside the scope of app::Roseate
    // so we can queue up notifications when things go wrong here.
    let notifier = NotifierAPI::new();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
        .with_inner_size([800.0, 600.0])
        .with_drag_and_drop(true),
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

            notifier.toasts.lock().unwrap().toast_and_log(
                error.into(), ToastLevel::Error
            ).duration(Some(Duration::from_secs(10)));
        } else {
            let result = image_handler.init_image(path);

            if let Err(error) = result {
                notifier.toasts.lock().unwrap().toast_and_log(
                    error.into(), ToastLevel::Error
                );
            }
        }
    }

    let theme = match theme_string {
        Some(string) => {
            if string == "light" {
                Theme::default(false)
            } else if string == "dark" {
                Theme::default(true)
            } else {
                log::warn!(
                    "'{}' is not a valid theme. Pass either 'dark' or 'light'.", string
                );

                Theme::default(true)
            }
        },
        _ => Theme::default(true)
    };

    let config = match Config::new() {
        Ok(config) => config,
        Err(error) => {

            notifier.toasts.lock().unwrap().toast_and_log(
                format!(
                    "Error occurred getting roseate's config file! \
                    Defaulting to default config. Error: {}", error.to_string().as_str()
                ).into(), 
                ToastLevel::Error
            ).duration(Some(Duration::from_secs(10)));

            Config::default()
        }
    };

    eframe::run_native(
        "Roseate",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Styling::new(&theme, None)
                .set_all()
                .apply(&cc.egui_ctx);

            Ok(Box::new(Roseate::new(image_handler, notifier, theme, config)))
        }),
    )
}