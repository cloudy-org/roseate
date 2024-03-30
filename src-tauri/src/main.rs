// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{env, sync::OnceLock};

use image::GenericImageView;

static _IMAGE: OnceLock<(String, (u32, u32))> = OnceLock::new();

#[tauri::command]
fn get_image() -> Option<(String, (u32, u32))> {
    _IMAGE.get().cloned()
}

fn main() {
    let cli_args: Vec<String> = env::args().collect();

    let image_path = cli_args.get(1);

    if image_path != None && !["", " "].contains(&image_path.unwrap().as_str()) {
        let path = image_path.unwrap().to_owned();

        let dimensions = image::open(&path).expect(
            "Failed to extract image dimensions!"
        ).dimensions();

        let _ = _IMAGE.set((path, dimensions));
    }

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_image])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}