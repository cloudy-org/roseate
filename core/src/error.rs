use std::{fmt::Display, io, result::Result as StdResult};

pub type Result<T, E = Error> = StdResult<T, E>;

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),

    UnsupportedColourType,

    ExifReaderImageMetadataParseFailure { error: String },

    DecodingFailure { error: String },
    DecoderInitFailure { error: String },
    DecoderRetrieveExifFailure { error: String },
    DecoderAnimationCheckFailure { error: String },
    DecoderImageFormatNotSupported { image_format: String, backend: String },

    ImageHeaderReadFailure { stage: String, error: Option<String> },
    ImageFormatNotSupported { image_format: String },
    ImageEncodeFailure { reason: String },

    AnimatedImageHasNoFrames,
}

// CError doesn't implement Display yet so I'm implementing it myself so 
// roseate's main error struct can just derive from this.
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::DecodingFailure { .. } => write!(
                f,
                "Decoder failed to decode the image! The image could be corrupted."
            ),
            Error::DecoderInitFailure { .. } => write!(
                f,
                "Failed to a initialize decoder!"
            ),
            Error::DecoderRetrieveExifFailure { .. } => write!(
                f,
                "Decoder failed to retrieve image exif chunk!"
            ),
            Error::DecoderAnimationCheckFailure { .. } => write!(
                f,
                "The backend's decoder unexpectedly failed to check if the image was animated!",
            ),
            Error::DecoderImageFormatNotSupported { image_format, backend } => write!(
                f,
                "The '{}' backend does not support the '{}' image format!",
                image_format,
                backend
            ),
            // NOTE: I think I'm gonna change things everywhere to only 
            // passing the actual error to "CError::actual_error()".
            Error::ImageHeaderReadFailure { stage, .. } => write!(
                f,
                "We failed to read the image's header, the image is \
                very likely corrupted! Try another image. Stage: {}",
                stage
            ),
            Error::ImageFormatNotSupported { image_format } => write!(
                f,
                "The image format '{image_format}' is not supported!"
            ),
            Error::ExifReaderImageMetadataParseFailure { ..} => write!(
                f,
                "Exif reader failed to parse image exif tags!"
            ),
            Error::ImageEncodeFailure { .. } => write!(
                f,
                "Failed to encode image, the image may be corrupted!"
            ),
            Error::AnimatedImageHasNoFrames => write!(
                f,
                "This animated image looks to be corrupted, it has no frames! \
                Are you sure this image is sound? Perhaps try another image."
            ),
            _ => todo!()
        }
    }
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