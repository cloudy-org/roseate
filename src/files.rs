use eframe::egui::{self, ImageSource};
use rfd::FileDialog;

use crate::error::Error;
use crate::image::image::Image;

pub fn select_image() -> Result<Image, Error> {
    let image_path = FileDialog::new()
        .add_filter("images", &["png", "jpeg", "jpg", "webp", "gif", "svg"])
        .pick_file();

    let image_or_error = match image_path {
        Some(path) => {
            if !path.exists() {
                Err(
                    Error::FileNotFound(
                        None,
                        path,
                        "The file picked in the file selector does not exist!".to_string()
                    )
                )
            } else {
                Image::from_path(&path)
            }
        },
        None => Err(Error::NoFileSelected(None))
    };

    image_or_error
}

pub fn get_platform_rose_image<'a>() -> ImageSource<'a> {
    if cfg!(target_os = "windows") {
        return egui::include_image!("../assets/rose_emojis/microsoft.png");
    } else if cfg!(target_os = "macos") {
        return egui::include_image!("../assets/rose_emojis/apple.png");
    }

    return egui::include_image!("../assets/rose_emojis/google_noto.png");
}