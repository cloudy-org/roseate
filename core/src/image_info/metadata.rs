use exif::{Field, In, Reader, Tag};
use log::debug;

use crate::error::{Error, Result};

#[derive(Default, Clone)]
pub struct Location {
    pub longitude: Option<String>,
    pub latitude: Option<String>,
    pub altitude: Option<String>
}

#[derive(Default, Clone)]
pub struct ImageMetadata {
    pub model: Option<String>,
    pub iso: Option<String>,
    pub aperture: Option<String>,
    pub focal_length: Option<String>,
    pub exposure_time: Option<String>,
    pub originally_created: Option<String>,

    pub location: Location,
}

impl ImageMetadata {
    pub fn new(exif_chunk: Vec<u8>) -> Result<Self> {
        debug!("Reading and parsing exif chunk...");

        // TODO: use the same buf reader we use for the decoder with
        // 'exif_reader.read_from_container' to save performance and reduce memory duplication.
        let exif_reader = Reader::new();
        let exif = exif_reader.read_raw(exif_chunk)
            .map_err(
                |error| Error::ExifReaderImageMetadataParseFailure {
                    error: error.to_string()
                }
            )?;

        let format_to_string_fn = |field: &Field| {
            field.display_value().to_string().trim_matches('"').to_string()
        };
        let to_option_fn = |field| Some(format_to_string_fn(field));

        let model_and_maker = match exif.get_field(Tag::Model, In::PRIMARY) {
            Some(model_field) => {
                let mut model = format_to_string_fn(model_field);

                if let Some(make_field) = exif.get_field(Tag::Make, In::PRIMARY) {
                    model += &format!(" ({})", format_to_string_fn(make_field));
                }

                Some(model)
            },
            None => None,
        };

        let exposure_time = match exif.get_field(Tag::ExposureTime, In::PRIMARY) {
            Some(field) => {
                match field.display_value().to_string().split_once(".") {
                    Some((whole_part, _)) => Some(whole_part.to_string()),
                    None => None,
                }
            },
            None => None,
        };

        let location = Location {
            longitude: exif.get_field(Tag::GPSLongitude, In::PRIMARY).and_then(to_option_fn),
            latitude: exif.get_field(Tag::GPSLatitude, In::PRIMARY).and_then(to_option_fn),
            altitude: exif.get_field(Tag::GPSAltitude, In::PRIMARY).and_then(to_option_fn)
        };

        Ok(
            Self {
                model: model_and_maker,
                iso: exif.get_field(Tag::PhotographicSensitivity, In::PRIMARY).and_then(to_option_fn),
                aperture: exif.get_field(Tag::ApertureValue, In::PRIMARY).and_then(to_option_fn),
                focal_length: exif.get_field(Tag::FocalLength, In::PRIMARY).and_then(to_option_fn),
                exposure_time: exposure_time,
                originally_created: exif.get_field(Tag::DateTimeOriginal, In::PRIMARY).and_then(to_option_fn),

                location: location,
            }
        )
    }
}
