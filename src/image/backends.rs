use std::fmt::Display;

pub enum ImageProcessingBackend {
    /// Uses the image-rs rust crate to do image processing and manipulation.
    /// This is by far the most stable backend.
    ImageRS,
    /// Uses Roseate's custom backend for image processing and manipulation.
    /// The Roseate backend is very fast for downsampling images but is VERY EXPERIMENTAL 
    /// and currently has wonky downsampling hence images may have weird artifacts.
    Roseate
}

impl Display for ImageProcessingBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageProcessingBackend::ImageRS => write!(f, "image-rs"),
            ImageProcessingBackend::Roseate => write!(f, "roseate"),
        }
    }
}