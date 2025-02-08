use std::{fmt::{self, Display, Formatter}, path::PathBuf, result::Result as StdResult};

type ActualError = Option<String>;
pub type Result<T, E = Error> = StdResult<T, E>;

#[derive(Debug, Clone)]
pub enum Error {
    FileNotFound(ActualError, PathBuf, String),
    NoFileSelected(ActualError),
    FailedToApplyOptimizations(ActualError, String),
    FailedToInitImage(ActualError, PathBuf, String),
    FailedToLoadImage(ActualError, String),
    ImageFormatNotSupported(ActualError, String),
    MonitorNotFound(ActualError),
    ImageFailedToEncode(ActualError, String),
    ImageFailedToDecode(ActualError, String),
}

impl Error {
    /// Returns the a human readable message about the error.
    /// It's exactly what fmt::Display returns.
    pub fn message(&self) -> String {
        format!("{}", self)
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
            Error::ImageFormatNotSupported(_, image_format) => write!(
                f, "The image format '{}' is not supported!", image_format
            ),
            Error::MonitorNotFound(_) => write!(
                f, "For some reason we couldn't detect your monitor.",
            ),
            Error::ImageFailedToEncode(_, technical_reason) => write!(
                f, "Image failed to encode! \n\nTechnical Reason: {}",
                technical_reason
            ),
            Error::ImageFailedToDecode(_, technical_reason) => write!(
                f, "Image failed to decode! \n\nTechnical Reason: {}",
                technical_reason
            ),
        }
    }
}