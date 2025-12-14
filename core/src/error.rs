use std::result::Result as StdResult;

pub type Result<T, E = Error> = StdResult<T, E>;

#[derive(Debug)]
pub enum Error {
    DecodingError,
    UnsupportedColourType,
    UnsupportedImageFormat,
    // FailureToInitDecoder,
}