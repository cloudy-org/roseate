use std::{fmt::{self, Display, Formatter}, path::PathBuf, result::Result as StdResult};

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