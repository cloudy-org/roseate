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