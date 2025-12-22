use std::fmt::Display;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ImageColourType {
    Grey8,
    Grey16,
    Grey32F,
    GreyA8,
    GreyA16,
    GreyA32F,
    Rgb8,
    Rgb16,
    Rgb32F,
    Rgba8,
    Rgba16,
    Rgba32F,
}

impl Display for ImageColourType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageColourType::Grey8 => write!(f, "Greyscale (8-bit)"),
            ImageColourType::Grey16 => write!(f, "Greyscale (16-bit)"),
            ImageColourType::Grey32F => write!(f, "Greyscale (32-bit float)"),
            ImageColourType::GreyA8 => write!(f, "Greyscale + Alpha (8-bit)"),
            ImageColourType::GreyA16 => write!(f, "Greyscale + Alpha (16-bit)"),
            ImageColourType::GreyA32F => write!(f, "Greyscale + Alpha (32-bit float)"),
            ImageColourType::Rgb8 => write!(f, "RGB (8-bit)"),
            ImageColourType::Rgb16 => write!(f, "RGB (16-bit)"),
            ImageColourType::Rgb32F => write!(f, "RGB (32-bit)"),
            ImageColourType::Rgba8 => write!(f, "RGBA (8-bit)"),
            ImageColourType::Rgba16 => write!(f, "RGBA (16-bit)"),
            ImageColourType::Rgba32F => write!(f, "RGBA (32-bit float)"),
        }
    }
}