use std::{result::Result as StdResult};

use cirrus_error::error::CError;

use roseate_core::{error::Error as CoreError, format::ImageFormat};
use cirrus_path::error::Error as PathError;
use cirrus_soft_binds::error::Error as SoftBindsError;

use derive_more::{Debug, Display, From};

pub type Result<T, E = Error> = StdResult<T, E>;

// I'm experimenting with "derive_more" for improved error handling all over the codebase.
// My end goal is to cover as many errors as possible with detailed messages for the end user.

#[derive(Display, Debug, From)]
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

    #[display("There is no backend available that supports the \
    '{image_format}' image format in this version of Roseate!")]
    BackendForImageFormatNotAvailable { image_format: ImageFormat },

    // External errors (some to later be handled better).
    #[from]
    Core(CoreError),

    #[from]
    PathError(PathError),
    #[from]
    SoftBinds(SoftBindsError),
    // #[from]
    // IO(io::Error),
}

impl CError for Error {}