use std::{path::PathBuf, sync::{Arc, Mutex}, thread};

use chrono::{DateTime, Local, NaiveDateTime};
use log::debug;
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

    pub fn start_location_lookup_thread(&mut self, image_metadata: &ImageMetadata) -> &mut Self {
        let location = self.location.clone();

        let image_location_latitude = image_metadata.location.latitude.clone();
        let image_location_longitude = image_metadata.location.longitude.clone();

        debug!("Spawning location lookup thread...");

        thread::spawn(move || {
            if let Some(latitude) = image_location_latitude
                && let Some(longitude) = image_location_longitude {
                // Locking at the beginning will tell the image info to display 
                // "Loading..." while the reverse geocoder initializes and finds the location.
                let mut location_mutex = location.lock().unwrap();

                debug!("Initializing reverse geocoder...");
                let geocoder = reverse_geocoder::ReverseGeocoder::new();

                debug!(
                    "Converting coordinates (latitude: {}, longitude: {}) to decimal...",
                    latitude, longitude,
                );

                let latitude = dms_to_decimal(&latitude);
                let longitude = dms_to_decimal(&longitude);

                let result = geocoder.search((latitude, longitude));

                debug!("Fetching image location country name...");

                if let Some(country_name) = country_emoji::name(&result.record.cc) {
                    let formatted_location = format!("{}, {}", result.record.name, country_name);

                    let url = format!(
                        "https://www.openstreetmap.org?mlat={}&mlon={}#map=18/{}/{}",
                        latitude, longitude, latitude, longitude
                    );

                    *location_mutex = Some((formatted_location, url));
                }
            }

        });

        self
    }
}

// NOTE: there was no benefit having this be a macro
fn dms_to_decimal(dms_string: &String) -> f64 {
    let parts: Vec<&str> = dms_string
        .split(|c: char| !c.is_ascii_digit() && c != '.' && c != '/')
        .filter(|s| !s.is_empty())
        .collect();

    let mut numbers = Vec::new();

    for part in parts {
        if part.contains("/") {
            let (first, second) = part.split_once("/").unwrap();
            if let (Ok(first), Ok(second)) = (
                first.parse::<f64>(),
                second.parse::<f64>()
            ) {
                numbers.push(first / second);
            } else {
                log::warn!("GPS Data format is unknown: {}", dms_string);
            }
        } else if let Ok(num) = part.parse::<f64>() {
            numbers.push(num);
        }
    }

    match numbers.len() {
        3 => numbers[0] + numbers[1] / 60.0 + numbers[2] / 3600.0,
        2 => numbers[0] + numbers[1] / 60.0,
        1 => numbers[0],
        _ => 0.0,
    }
}