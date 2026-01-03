use std::{path::PathBuf, sync::{Arc, Mutex}};

use log::debug;
use chrono::{DateTime, Local, NaiveDateTime};
use roseate_core::image_info::metadata::ImageMetadata;

use crate::{image_handler::resource::ImageResource};

pub struct ExpensiveData {
    pub file_name: String,
    pub file_size: Option<f64>,
    pub file_relative_path: String,
    pub image_created_time: Option<String>,
    pub file_modified_time: Option<String>,
    pub memory_allocated_for_image: f64,

    pub location: Arc<Mutex<Option<(String, String)>>>
}

impl ExpensiveData {
    pub fn new(image_path: &Arc<PathBuf>, image_resource: &ImageResource, image_metadata: &ImageMetadata) -> Self {
        let file_name = image_path.file_name().unwrap().to_string_lossy().to_string();
        let file_relative_path = image_path.to_string_lossy().to_string();

        let file_metadata = match image_path.metadata() {
            Ok(metadata) => Some(metadata),
            Err(error) => {
                log::error!(
                    "Failed to retrive image file metadata from file system! Error: {}",
                    error
                );

                None
            },
        };

        let mut file_size = None;
        let mut image_created_time = None;
        let mut file_modified_time = None;

        let date_format = "%d/%m/%Y %H:%M %p";

        if let Some(metadata) = file_metadata {
            image_created_time = match metadata.created() {
                Ok(time) => {
                    let datetime: DateTime<Local> = time.into();
                    Some(datetime.format(date_format).to_string())
                },
                Err(error) => {
                    log::warn!("Failed to retrieve image file created date! Error: {}", error);

                    None
                },
            };

            file_modified_time = match metadata.modified() {
                Ok(time) => {
                    let datetime: DateTime<Local> = time.into();
                    Some(datetime.format(date_format).to_string())
                },
                Err(error) => {
                    log::warn!("Failed to retrieve image file modified date! Error: {}", error);

                    None
                },
            };

            file_size = Some(metadata.len() as f64);
        }

        if let Some(image_original_creation_time) = &image_metadata.originally_created {
            debug!("Parsing image original creation date...");
            match NaiveDateTime::parse_from_str(image_original_creation_time, "%Y-%m-%d %H:%M:%S") {
                Ok(datetime) => {
                    image_created_time = Some(datetime.format(date_format).to_string());
                },
                Err(error) => {
                    log::warn!("Failed to parse image original creation date! Error: {}", error);
                }
            }
        }

        Self {
            file_name,
            file_size,
            file_relative_path,
            image_created_time,
            file_modified_time,
            memory_allocated_for_image: match image_resource {
                ImageResource::Texture(texture_handle) => texture_handle.byte_size() as f64,
                ImageResource::AnimatedTexture(frames) => {
                    let mut size = 0;

                    for (texture_handler, _) in frames {
                        size += texture_handler.byte_size();
                    }

                    size as f64
                },
            },

            location: Arc::new(Mutex::new(None))
        }
    }
}

