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

pub fn get_cache_path() -> Result<PathBuf> {
    debug!("Finding operating system's cache directory...");

    let cache_dir = match dirs::cache_dir() {
        Some(cache_dir) => cache_dir.join("cloudy").join("roseate"),
        None => {
            return Err(Error::CacheDirectoryNotFound)
        }
    };

    if !cache_dir.exists() {
        debug!("Creating cache directory for roseate at '{}'...", cache_dir.to_string_lossy());

        if let Err(error) = fs::create_dir_all(&cache_dir) {
            return Err(
                Error::CacheDirectoryCreationFailure {
                    path: cache_dir.to_string_lossy().to_string(),
                    error: error.to_string()
                }
            );
        };

        debug!("Cache directory created ({})!", cache_dir.to_string_lossy());
    }

    Ok(cache_dir)
}