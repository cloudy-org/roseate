pub mod image;
pub mod key_binds;
pub mod ui;
pub mod misc;
pub mod image_optimizations;

pub fn true_default() -> bool {
    true
}

pub fn false_default() -> bool {
    false
}

pub fn none_default<T>() -> Option<T> {
    None
}