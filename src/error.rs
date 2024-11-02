use std::{fmt::{self, Display, Formatter}, path::PathBuf};

#[derive(Debug)]
pub enum Error {
    FileNotFound(PathBuf),
    NoFileSelected
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Error::FileNotFound(path) => write!(
                f, "The file path given '{}' does not exist!", path.to_string_lossy()
            ),
            Error::NoFileSelected => write!(
                f, "No file was selected in the file dialogue!"
            )
        }
    }
}