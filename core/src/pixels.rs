use std::{fmt::Display, ops::{Deref, DerefMut}};

use crate::colour_type::ImageColourType;

#[derive(Debug)]
pub enum Pixels {
    U8(Vec<u8>),
    U16(Vec<u16>),
    F32(Vec<f32>)
}

impl Display for Pixels {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Pixels::U8(_) => write!(f, "pixels (with u8 bit depth)"),
            Pixels::U16(_) => write!(f, "pixels (with u16 bit depth)"),
            Pixels::F32(_) => write!(f, "pixels (with f32 bit depth)"),
        }
    }
}

impl Pixels {
    pub fn new(colour_type: &ImageColourType, buffer_size: usize) -> Self {
        match colour_type {
            ImageColourType::Grey8 | 
            ImageColourType::GreyA8 | 
            ImageColourType::Rgb8 | 
            ImageColourType::Rgba8 => {
                Self::U8(vec![0; buffer_size])
            },
            ImageColourType::Grey16 |
            ImageColourType::GreyA16 |
            ImageColourType::Rgb16 |
            ImageColourType::Rgba16 => {
                Self::U16(vec![0; buffer_size / 2])
            },
            ImageColourType::Grey32F |
            ImageColourType::GreyA32F |
            ImageColourType::Rgb32F |
            ImageColourType::Rgba32F => {
                Self::F32(vec![0.0; buffer_size / 4])
            },
        }
    }
}

impl Deref for Pixels {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        match self {
            Pixels::U8(pixels) => pixels.as_slice(),
            Pixels::U16(pixels) => bytemuck::cast_slice(pixels.as_slice()),
            Pixels::F32(pixels) => bytemuck::cast_slice(pixels.as_slice()),
        }
    }
}

impl DerefMut for Pixels {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Pixels::U8(pixels) => pixels.as_mut_slice(),
            Pixels::U16(pixels) => bytemuck::cast_slice_mut(pixels.as_mut_slice()),
            Pixels::F32(pixels) => bytemuck::cast_slice_mut(pixels.as_mut_slice()),
        }
    }
}