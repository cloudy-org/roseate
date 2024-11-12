use std::{fmt::{self, Display, Formatter}, path::PathBuf, time::Duration};

use egui_notify::{Toast, Toasts};

#[derive(Debug)]
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

pub enum LogAndToastError {
    Error(Error),
    String(String)
}

impl Into<LogAndToastError> for Error {
    fn into(self) -> LogAndToastError {
        LogAndToastError::String(self.message())
    }
}

impl Into<LogAndToastError> for String {
    fn into(self) -> LogAndToastError {
        LogAndToastError::String(self)
    }
}

impl Into<LogAndToastError> for &str {
    fn into(self) -> LogAndToastError {
        LogAndToastError::String(self.to_string())
    }
}

pub fn log_and_toast(error_or_string: LogAndToastError, toasts: &mut Toasts) -> &mut Toast {
    let error_message = match error_or_string {
        LogAndToastError::Error(error) => error.message(),
        LogAndToastError::String(string) => string,
    };

    log::error!("{}", error_message);

    toasts.error(
        textwrap::wrap(error_message.as_str(), 100).join("\n")
    ).duration(Some(Duration::from_secs(5)))
}