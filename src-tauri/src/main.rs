// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use rfd::AsyncFileDialog;
use image::ImageFormat;
use serde::{de::IntoDeserializer, Deserialize};
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
    tauri::Builder::default()
        .plugin(cirrus_tauri::init())
        .setup(|app| {
            match app.get_cli_matches() {
                Ok(matches) => {
                    let source = matches.args.get("source").unwrap();

                    if !source.value.is_boolean() { // if tauri decides to return a 'true' boolean here SHIT WILL BLOW UP!!!
                        let source: Result<Option<String>, serde_json::Error> = empty_string_as_none(&source.value);
                        let source = source.expect("Source must be a string!");

                        if !source.is_none() {
                            let image_path = source.unwrap();
                            set_image(&image_path.to_string());
                        }
                    }
                }
                Err(_) => {}
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_image, select_image, set_image_drag_drop])
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

#[tauri::command]
fn set_image_drag_drop(path: Vec<String>) {
    let first_item = path.get(0).unwrap();

    set_image(first_item);
}

fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    let opt = Option::<String>::deserialize(de)?;
    let opt = opt.as_ref().map(String::as_str);
    match opt {
        None | Some("") => Ok(None),
        Some(s) => T::deserialize(s.into_deserializer()).map(Some)
    }
}