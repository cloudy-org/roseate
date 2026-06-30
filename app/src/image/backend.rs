use std::fmt::Display;

use cirrus_egui::notifier::Notifier;
use egui_notify::ToastLevel;
use log::{debug, warn};
use roseate_core::{backends::{backend::DecodeBackend, image_rs::ImageRSBackend}, format::ImageFormat, reader::ImageReader};

use crate::error::{Error, Result};

#[derive(Clone, Ord, Eq, PartialEq, PartialOrd)]
pub enum DefaultDecodingBackend {
    /// Uses the image-rs rust crate for image decoding and 
    /// modifications. This is by far the most stable backend.
    ImageRS,
    /// Uses the zune-image rust crate for image decoding and modifications, 
    /// it's fast but it's implementation in roseate is currently experimental.
    ZuneImage,
}

impl Display for DefaultDecodingBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DefaultDecodingBackend::ImageRS => write!(f, "image-rs"),
            DefaultDecodingBackend::ZuneImage => write!(f, "zune-image"),
        }
    }
}

impl DefaultDecodingBackend {
    const PRIORITIZED_BACKENDS: [Self; 2] = [Self::ImageRS, Self::ZuneImage];

    pub fn init_default_backend_or_fallback_if_not_supported(
        &self,
        image_reader: ImageReader,
        notifier: &mut Notifier,
        fallback_on_unsupported_image_format: bool,
    ) -> Result<impl DecodeBackend + use<>> {
        let mut prioritized_backends = vec![self.clone()];

        if fallback_on_unsupported_image_format {
            for backend in Self::PRIORITIZED_BACKENDS {
                if &backend != self {
                    prioritized_backends.push(backend);
                }
            }
        }

        for backend in prioritized_backends {
            if backend.is_image_format_supported(&image_reader.image_format) {
                // TODO: if decoder fails to init, also fallback to the next decoder
                return backend.init_backend_decoder(image_reader);
            }

            notifier.toast(
                format!(
                    "The decoder backend '{backend}' does not support \
                    this image format, falling back to another backend..."
                ),
                ToastLevel::Warning,
                |_| {}
            );
        }

        Err(
            Error::BackendForImageFormatNotAvailable {
                image_format: image_reader.image_format
            }
        )
    }

    fn init_backend_decoder(self, image_reader: ImageReader) -> Result<impl DecodeBackend> {
        debug!("Initializing '{self}' backend decoder...");

        let decoder = match self {
            Self::ImageRS => ImageRSBackend::from_reader(image_reader)?,
            Self::ZuneImage => todo!("The ZuneImage backend is not yet implemented in roseate-core!"),
        };

        Ok(decoder)
    }

    fn is_image_format_supported(&self, image_format: &ImageFormat) -> bool {
        match self {
            Self::ImageRS => ImageRSBackend::SUPPORTED_FORMATS.contains(&image_format),
            Self::ZuneImage => false,
        }
    }
}