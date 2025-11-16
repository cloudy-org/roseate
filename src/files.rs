use log::debug;
use std::{fs, path::PathBuf};

use eframe::egui::{self, ImageSource};

use crate::error::{Error, Result};

// TODO: add ability to customize this image in the future
pub fn get_rose_image<'a>() -> ImageSource<'a> {
    if cfg!(target_os = "windows") {
        return egui::include_image!("../assets/rose_emojis/microsoft.png");
    }

    return egui::include_image!("../assets/rose_emojis/google_noto.png");
}

// TODO: move get path functionality to cirrus.

// TODO: make this return result and also handle creating path if it doesn't exist
// just like the function below basically.
pub fn get_local_config_path() -> Option<PathBuf> {
    debug!("Finding operating system's configuration local directory...");

    match dirs::config_local_dir() {
        Some(local_config_dir) => Some(
            local_config_dir.join("cloudy").join("roseate")
        ),
        None => None
    }
}

pub fn get_cache_path() -> Result<PathBuf> {
    debug!("Finding operating system's cache directory...");

    let cache_dir = match dirs::cache_dir() {
        Some(cache_dir) => cache_dir.join("cloudy").join("roseate"),
        None => {
            return Err(Error::OSDirNotFound(None, "cache".into()))
        }
    };

    if !cache_dir.exists() {
        debug!("Creating cache directory for roseate at '{}'...", cache_dir.to_string_lossy());

        if let Err(error) = fs::create_dir_all(&cache_dir) {
            return Err(
                Error::FailedToCreatePath(Some(error.to_string()), cache_dir)
            );
        };

        debug!("Cache directory created ({})!", cache_dir.to_string_lossy());
    }

    Ok(cache_dir)
}