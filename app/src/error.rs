use std::{io, result::Result as StdResult};

use cirrus_error::v1::error::CError;
use roseate_core::error::Error as CoreError;
use cirrus_softbinds::v1::error::Error as SoftBindsError;

use derive_more::{Debug, Display, From};

pub type Result<T, E = Error> = StdResult<T, E>;

// I'm experimenting with "derive_more" for improved error handling all over the codebase.
// My end goal is to cover as many errors as possible with detailed messages for the end user.

#[derive(Debug, Display, From)]
pub enum Error {
    #[display("The image file at '{path}' does not exist!")]
    FileNotFound { path: String },

    #[display("No image was selected in the file dialogue!")]
    FileNotSelected,

    #[display("We failed to open the image file for reading! \n\nError: {error}")]
    ImageFileOpenFailure { error: String },

    // #[display("We failed to perform optimizations on this image! \
    // Roseate may run slower and use more memory than usual!")]
    // ImageOptimizationFailure { reason: String },

    // NOTE: this error will be removed soon.
    #[display("No cache directory was found for your Operating \
    System!? This should not happen, please report this!")]
    CacheDirectoryNotFound,

    // NOTE: same thing with this error, will also be removed soon.
    #[display("Failed to create cache path at '{path}'! \n\nError: {error}")]
    CacheDirectoryCreationFailure { path: String, error: String },

    // NOTE: anddddd.... this
    #[display("Failed to read the '{file_name}' cache file! \n\nError: {error}")]
    CacheFileReadFailure { file_name: String, error: String },

    #[display("Experimental SVG support has been temporary removed from \
    Roseate! We're still working on SVG support, it will be back when it's ready.")]
    SvgNotSupportedYet,

    // External errors (some to later be handled better).
    #[from]
    Core(CoreError),

    #[from]
    SoftBinds(SoftBindsError),
    // #[from]
    // IO(io::Error),
}

impl CError for Error {
    fn human_message(&self) -> String {
        // NOTE: I plan to move to the Display trait for a "human message".
        format!("{}", self)
    }

    // TODO: Add some sort of button to the our notifier toast 
    // that allows users to see the more verbose error message (debug) in the GUI.
    fn actual_error(&self) -> Option<String> {
        match self {
            // Error::ImageOptimizationFailure { reason } => Some(reason.into()),
            Error::Core(error) => match error {
                CoreError::ImageHeaderReadFailure { error, .. } => error.to_owned(),
                CoreError::ImageEncodeFailure { reason } => Some(reason.into()),
                _ => None
            }
            // Error::IO(error) => todo!(),
            _ => None
        }
    }
}