use std::{fmt::{self, Display, Formatter}, path::PathBuf};

#[derive(Debug, Clone)]
pub enum Error {
    FileNotFound(PathBuf),
    NoFileSelected,
    FailedToApplyOptimizations(String),
    ImageFormatNotSupported(String),
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
            Error::FileNotFound(path) => write!(
                f, "The file path given '{}' does not exist!", path.to_string_lossy()
            ),
            Error::NoFileSelected => write!(
                f, "No file was selected in the file dialogue!"
            ),
            Error::FailedToApplyOptimizations(technical_reason) => write!(
                f,
                "Failed to apply optimizations to this image! \
                    Roseate will run slower than usual and use a lot more memory \
                    possibly leading to system crashes. BEWARE! \n\nTechnical Reason: {}",
                technical_reason
            ),
            Error::ImageFormatNotSupported(image_format) => write!(
                f, "The image format '{}' is not supported!", image_format
            ),
        }
    }
}