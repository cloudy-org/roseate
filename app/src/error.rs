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

    #[display("We failed to open the image file for reading!")]
    ImageFileOpenFailure { error: String },

    #[display("Failed to create cache path at '{path}'!")]
    CacheDirectoryCreationFailure { path: String, error: String },

    #[display("Failed to get and read monitor size cache file!")]
    GetCachedMonitorSizeFailure { error: String },
    #[display("Failed to write monitor size to cache file!")]
    WriteCachedMonitorSizeFailure { error: String },
    #[display("The 'monitor_size' cache file is currently locked by another Roseate \
    instant, so we will have to wait until the other instance let's go of \
    the file until we can save our monitor size state to it.")]
    CachedMonitorSizeAlreadyLocked { error: String },

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