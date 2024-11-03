use std::{fmt::{self, Display, Formatter}, path::PathBuf, time::Duration};

use egui_notify::{Toast, Toasts};

#[derive(Debug)]
pub enum Error {
    FileNotFound(PathBuf),
    NoFileSelected
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
        }
    }
}

pub fn log_and_toast(error: Error, toasts: &mut Toasts) -> &mut Toast {
    log::error!("{}", error);

    toasts.error(error.message())
        .duration(Some(Duration::from_secs(5)))
}