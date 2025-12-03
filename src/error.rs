use std::{fmt::{self, Display, Formatter}, path::PathBuf, result::Result as StdResult};

use cirrus_error::v1::error::CError;

type ActualError = Option<String>;
pub type Result<T, E = Error> = StdResult<T, E>;

#[derive(Debug, Clone)]
pub enum Error {
    FileNotFound(ActualError, PathBuf, String),
    NoFileSelected(ActualError),
    FailedToApplyOptimizations(ActualError, String),
    FailedToInitImage(ActualError, PathBuf, String),
    /// String: user friendly reason why image didn't load
    FailedToLoadImage(ActualError, String),
    /// String: technical reason to why the image failed to convert to pixels.
    FailedToLoadTexture(ActualError),
    FailedToConvertImageToPixels(ActualError, String),
    ImageFormatNotSupported(ActualError, String),
    MonitorNotFound(ActualError),
    FailedToEncodeImage(ActualError, String),
    FailedToDecodeImage(ActualError, String),
    OSDirNotFound(ActualError, String),
    /// PathBuf: the path that failed to be created
    FailedToCreatePath(ActualError, PathBuf),
    /// PathBuf: the path to the file that failed to open
    FailedToOpenFile(ActualError, PathBuf)
}

impl CError for Error {
    fn human_message(&self) -> String {
        // the display implementation code was there way before CError in cirrus became a thing.
        format!("{}", self)
    }

    fn actual_error(&self) -> Option<String> {
        match self {
            Error::FileNotFound(actual_error, _, _) => actual_error,
            Error::NoFileSelected(actual_error) => actual_error,
            Error::FailedToApplyOptimizations(actual_error, _) => actual_error,
            Error::FailedToInitImage(actual_error, _, _) => actual_error,
            Error::FailedToLoadImage(actual_error, _) => actual_error,
            Error::FailedToLoadTexture(actual_error) => actual_error,
            Error::FailedToConvertImageToPixels(actual_error, _) => actual_error,
            Error::ImageFormatNotSupported(actual_error, _) => actual_error,
            Error::MonitorNotFound(actual_error) => actual_error,
            Error::FailedToEncodeImage(actual_error, _) => actual_error,
            Error::FailedToDecodeImage(actual_error, _) => actual_error,
            Error::OSDirNotFound(actual_error, _) => actual_error,
            Error::FailedToCreatePath(actual_error, _) => actual_error,
            Error::FailedToOpenFile(actual_error, _) => actual_error,
        }.to_owned()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Error::FileNotFound(_, path, detail) => {
                let message = format!(
                    "The file path given '{}' does not exist! {}",
                    path.to_string_lossy(),
                    detail
                );

                write!(f, "{}", message)
            },
            Error::NoFileSelected(_) => write!(
                f, "No file was selected in the file dialogue!"
            ),
            Error::FailedToApplyOptimizations(_, technical_reason) => write!(
                f,
                "Failed to apply optimizations to this image! \
                    Roseate will run slower than usual and use a lot more memory \
                    possibly leading to system crashes. BEWARE! \n\nTechnical Reason: {}",
                technical_reason
            ),
            Error::FailedToInitImage(_, path, reason) => write!(
                f,
                "Failed to initialize the image ({})! Reason: {}",
                path.file_name().unwrap().to_string_lossy(),
                reason
            ),
            Error::FailedToLoadImage(_, reason) => write!(
                f,
                "Failed to load that image! The image might be corrupted. Reason: {}",
                reason
            ),
            Error::FailedToLoadTexture(_) => write!(
                f, "Egui failed to load image texture! Possible image corruption."
            ),
            Error::FailedToConvertImageToPixels(_, technical_reason) => write!(
                f,
                "Failed to transform image to pixels! The image may be corrupted. Technical Reason: {}",
                technical_reason
            ),
            Error::ImageFormatNotSupported(_, image_format) => write!(
                f, "The image format '{}' is not supported!", image_format
            ),
            Error::MonitorNotFound(_) => write!(
                f, "For some reason we couldn't detect your monitor.",
            ),
            Error::FailedToEncodeImage(_, technical_reason) => write!(
                f,
                "Image failed to encode! \n\nTechnical Reason: {}",
                technical_reason
            ),
            Error::FailedToDecodeImage(_, technical_reason) => write!(
                f,
                "Image failed to decode! \n\nTechnical Reason: {}",
                technical_reason
            ),
            Error::OSDirNotFound(_, directory_name) => write!(
                f,
                "No '{}' directory was found for your Operating System!? \
                    This should not happen, please report this!",
                directory_name
            ),
            Error::FailedToCreatePath(_, path) => write!(
                f,
                "Failed to create path at '{}'!",
                path.to_string_lossy()
            ),
            Error::FailedToOpenFile(_, path) => write!(
                f,
                "Failed to open file at '{}'!",
                path.to_string_lossy()
            )
        }
    }
}