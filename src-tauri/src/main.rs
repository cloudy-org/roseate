// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use rfd::AsyncFileDialog;
use image::ImageFormat;
use std::{env, sync::OnceLock};

static _IMAGE: OnceLock<(String, (usize, usize))> = OnceLock::new();

#[tauri::command]
fn get_image() -> Option<(String, (usize, usize))> {
    _IMAGE.get().cloned()
}

#[tauri::command(async)]
async fn select_image() {
    set_image(&pick_image().await.to_str().unwrap().to_string());
}

fn main() {
    let cli_args: Vec<String> = env::args().collect();

    let image_path = cli_args.get(1);

    if image_path != None && !["", " "].contains(&image_path.unwrap().as_str()) {
        set_image(&image_path.unwrap().to_owned());
    }

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_image, select_image])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

pub async fn pick_image() -> PathBuf {
    let mut extensions_list: Vec<&str> = vec![];

    for image_format in ImageFormat::all() {
        extensions_list.extend(image_format.extensions_str())
    }

    let file_handle = AsyncFileDialog::new()
        .set_title("Choose a text file...")
        .add_filter("Image", &extensions_list)
        .pick_file()
        .await
        .expect("Failed to pick file from the file dialog!");

    file_handle.path().to_owned()
}

fn set_image(path: &String) {
    let image_result = imagesize::size(path).expect(
        &format!("Failed to retrieve size of the image '{}'", path)
    );

    let dimensions = (image_result.width, image_result.height);

    let _ = _IMAGE.set((path.to_owned(), dimensions));
}