#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::{env, path::Path, time::Duration};

use cirrus_egui::v1::styling::Styling;
use config::config::Config;
use image_handler::{optimization::ImageOptimizations, ImageHandler};
use log::debug;
use eframe::egui;
use egui_notify::ToastLevel;
use cirrus_theming::v1::{Colour, Theme};
use clap::{arg, command, Parser};

use error::Error;
use app::Roseate;
use monitor_size::MonitorSize;
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
mod monitor_size;

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
        notifier.toasts.lock().unwrap()
            .toast_and_log(
                "The monitor size was not cached yet so the \
                image MAY appear a little blurry or over sharpened at first. Roseate will \
                clear this up and this should never happen again the next time you launch Roseate.".into(),
                ToastLevel::Warning
            ).duration(Some(Duration::from_secs(10)));
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

            notifier.toasts.lock().unwrap().toast_and_log(
                error.into(), ToastLevel::Error
            ).duration(Some(Duration::from_secs(10)));
        } else {
            let mut configured_image_optimizations = config.image.optimizations.get_optimizations();

            // TODO: remove this once we move DS to image.optimizations.
            if config.misc.experimental.use_dynamic_sampling_optimization {
                configured_image_optimizations.push(
                    ImageOptimizations::DynamicSampling(true, true)
                );
            }

            let result = image_handler.init_image(path, configured_image_optimizations);

            if let Err(error) = result {
                notifier.toasts.lock().unwrap().toast_and_log(
                    error.into(), ToastLevel::Error
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

            Ok(Box::new(Roseate::new(image_handler, monitor_size, notifier, theme, config)))
        }),
    )
}