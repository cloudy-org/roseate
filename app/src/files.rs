use eframe::egui::{self, ImageSource};

// TODO: add ability to customize this image in the future
pub fn get_rose_image<'a>() -> ImageSource<'a> {
    if cfg!(target_os = "windows") {
        return egui::include_image!("../assets/rose_emojis/microsoft.png");
    }

    return egui::include_image!("../assets/rose_emojis/google_noto.png");
}