use std::{path::PathBuf};

use rfd::FileDialog;
use roseate_core::format::IMAGE_FORMAT_EXTENSIONS;

use crate::{error::{Error, Result}, image::Image};

// TODO: this struct should go under app.rs folder module when that exists 

#[derive(Default)]
pub struct ImageSelector {
    // TODO: switch Image to Arc<Image>.
    selected: usize,

    images: Vec<Image>,
}

impl ImageSelector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_image(&self) -> Option<&Image> {
        self.images.get(self.selected)
    }

    pub fn get_mutable_image(&mut self) -> Option<&mut Image> {
        self.images.get_mut(self.selected)
    }

    pub fn select(&mut self, index: usize) {
        log::debug!("Selecting image at index '{index}'...");

        let max_index = self.images.len() - 1;

        if index > max_index {
            log::debug!(
                "No image exists at index '{index}' to select! Selecting last image..."
            );
            self.selected = max_index;
        }

        self.selected = index;
    }

    pub fn select_image_from_path(&mut self, path: PathBuf) -> Result<()> {
        for (index, image) in self.images.iter().enumerate() {
            if *image.path == path {
                self.select(index);

                return Ok(())
            }
        }

        self.images.push(Image::new(path)?);

        self.select(self.images.len() - 1);

        Ok(())
    }

    pub fn select_image_from_file_explorer(&mut self) -> Result<()> {
        let image_path = FileDialog::new()
            .add_filter("images", IMAGE_FORMAT_EXTENSIONS)
            .pick_file();

        match image_path {
            Some(path) => self.select_image_from_path(path),
            None => Err(Error::FileNotSelected)
        }
    }
}