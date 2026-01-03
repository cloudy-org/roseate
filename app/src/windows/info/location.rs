use log::debug;
use std::thread;
use roseate_core::image_info::metadata::ImageMetadata;

use crate::windows::info::expensive_data::ExpensiveData;

impl ExpensiveData {
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