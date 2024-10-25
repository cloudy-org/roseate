#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::{path::Path, process::exit};

use app::Roseate;
use log::debug;
use eframe::egui;
use clap::{arg, command, Parser};
use cirrus_theming::Theme;

use image::Image;

mod app;
mod image;
mod info_box;
mod zoom_pan;
mod window_scaling;

/// 🌹 A small and simple but fancy image viewer built with Rust that's cross-platform.
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
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    let cli_args = Args::parse();

    let image_path = cli_args.image;
    let theme_string = cli_args.theme;

    if image_path.is_some() {
        debug!("Image '{}' loading from path...", &image_path.as_ref().unwrap());
    }

    let image = match image_path {
        Some(path) => {
            let path = Path::new(&path);

            if !path.exists() {
                error_then_exit(
                    &format!("The file path given '{}' does not exist!", path.to_string_lossy()), 1
                );
            }

            Some(Image::from_path(path))
        },
        None => None
    };

    let theme = match theme_string {
        Some(string) => {
            if string == "light" {
                Theme::default(false)
            } else if string == "dark" {
                Theme::default(true)
            } else {
                error_then_exit(
                    &format!("'{}' is not a valid theme. Pass either 'dark' or 'light'.", string), 1
                );
            }
        },
        _ => Theme::default(true)
    };

    eframe::run_native(
        "Roseate",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(Roseate::new(image, theme)))
        }),
    )
}

fn error_then_exit(message: &str, exit_code: i32) -> ! {
    println!("\u{001b}[31;1mERROR:\u{001b}[0m {}", message);
    exit(exit_code)
}