use std::fmt::Display;

#[derive(Clone, Debug, PartialEq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Svg,
    Gif,
    Webp
}

impl Display for ImageFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageFormat::Png => write!(f, "PNG (Portable Network Graphics)"),
            ImageFormat::Jpeg => write!(f, "JPEG (Joint Photographic Experts Group)"),
            ImageFormat::Svg => write!(f, "SVG (Scalable Vector Graphics)"),
            ImageFormat::Gif => write!(f, "GIF (Graphics Interchange Format)"),
            ImageFormat::Webp => write!(f, "WEBP (Web Picture)"),
        }
    }
}