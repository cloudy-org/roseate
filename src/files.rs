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
                Err(Error::FileNotFound(path, None))
            } else {
                Ok(Image::from_path(&path))
            }
        },
        None => Err(Error::NoFileSelected)
    };

    image_or_error
}