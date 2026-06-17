use std::{path::PathBuf};

use eframe::egui::ahash::HashMap;
use rfd::FileDialog;

use crate::{error::{Error, Result}, image::Image};

// TODO: this struct should go under app.rs folder module when that exists 

#[derive(Default)]
pub struct ImageSelector {
    // TODO: switch Image to Arc<Image>.
    selected: usize,

    images: HashMap<usize, Image>,
}

impl ImageSelector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_image(&self) -> Option<&Image> {
        self.images.get(&self.selected)
    }

    pub fn get_mutable_image(&mut self) -> Option<&mut Image> {
        self.images.get_mut(&self.selected)
    }

    pub fn select_image_from_path(&mut self, path: PathBuf) -> Result<()> {
        // NOTE: we don't support loading multiple images yet 
        // so we're not gonna handle any of that logic yet here 
        self.images.insert(0, Image::new(path)?);

        Ok(())
    }

    pub fn select_image_from_file_explorer(&mut self) -> Result<()> {
        let image_path = FileDialog::new()
            .add_filter("images", &["png", "jpeg", "jpg", "webp", "gif", "svg"])
            .pick_file();

        match image_path {
            Some(path) => self.select_image_from_path(path),
            None => Err(Error::FileNotSelected)
        }
    }
}