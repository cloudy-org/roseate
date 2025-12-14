use std::{io, result::Result as StdResult};

use image::ImageError;

pub type Result<T, E = Error> = StdResult<T, E>;

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    DecodingError(String),
    AnimationCheckError(String),
    UnsupportedColourType,
    UnsupportedImageFormat,

    UnsupportedDecoder(String),
}

// impl From<ImageError> for Error {
//     fn from(value: ImageError) -> Self {
//         match value {
//             // TODO: specify more info where relevant
//             ImageError::Decoding(decoding_error) => Self::DecodingError(decoding_error.to_string()),
//             ImageError::Encoding(encoding_error) => unreachable!(),
//             ImageError::Parameter(parameter_error) => unreachable!(),
//             ImageError::Limits(limit_error) => Self::DecodingError(limit_error.to_string()),
//             // TODO: Handle Unsupported kind and don't just pass string.
//             ImageError::Unsupported(unsupported_error) => Self::UnsupportedDecoder(unsupported_error.to_string()),
//             ImageError::IoError(error) => Self::IOError(error),
//         }
//     }
// }