use std::{io::Cursor};

use image::{Rgb, Rgba};
use roseate_core::{self, backends::{backend::DecodeBackend, image_rs::ImageRSBackend}, error::Result, format::ImageFormat, colour_type::ImageColourType, modifications::ImageModification, reader::ImageReader};

use crate::backends::{save_image};

#[test]
fn init() {
    env_logger::init();
}

#[test]
fn test_png_decode_1() -> Result<()> {
    let image_bytes = include_bytes!("../mov_cli_logo.png");

    let cursor = Cursor::new(&image_bytes[..]);
    let image_reader = ImageReader::new(cursor, ImageFormat::Png);

    let backend = ImageRSBackend::from_reader(image_reader)?;
    let decoded_image = backend.decode()?;

    assert_eq!(decoded_image.size, (750, 250));
    assert_eq!(decoded_image.colour_type, ImageColourType::Rgba8);

    save_image::<Rgba<u8>>(decoded_image, "mov-cli.png");

    Ok(())
}

#[test]
fn test_png_decode_2() -> Result<()> {
    let image_bytes = include_bytes!("../mia_holding_rust_book.png");

    let cursor = Cursor::new(&image_bytes[..]);
    let image_reader = ImageReader::new(cursor, ImageFormat::Png);

    let backend = ImageRSBackend::from_reader(image_reader)?;
    let decoded_image = backend.decode()?;

    assert_eq!(decoded_image.size, (1920, 1080));
    assert_eq!(decoded_image.colour_type, ImageColourType::Rgba8);

    save_image::<Rgba<u8>>(decoded_image, "normal_mia.png");

    Ok(())
}

#[test]
fn test_png_modify_and_decode_1() -> Result<()> {
    let image_bytes = include_bytes!("../mia_holding_rust_book.png");

    let cursor = Cursor::new(&image_bytes[..]);
    let image_reader = ImageReader::new(cursor, ImageFormat::Png);

    let mut backend = ImageRSBackend::from_reader(image_reader)?;
    backend.modify(vec![ImageModification::Resize(1280, 720)]);

    let decoded_image = backend.decode()?;

    assert_eq!(decoded_image.size, (1280, 720));
    assert_eq!(decoded_image.colour_type, ImageColourType::Rgba8);

    save_image::<Rgba<u8>>(decoded_image, "smaller_mia.png");

    Ok(())
}

#[test]
fn test_png_modify_and_decode_2() -> Result<()> {
    let image_bytes = include_bytes!("../mia_holding_rust_book.png");

    let cursor = Cursor::new(&image_bytes[..]);
    let image_reader = ImageReader::new(cursor, ImageFormat::Png);

    let mut backend = ImageRSBackend::from_reader(image_reader)?;
    backend.modify(vec![ImageModification::Resize(540, 270)]);

    let decoded_image = backend.decode()?;

    assert_eq!(decoded_image.size, (540, 270));
    assert_eq!(decoded_image.colour_type, ImageColourType::Rgba8);

    save_image::<Rgba<u8>>(decoded_image, "tiny_mia.png");

    Ok(())
}

#[test]
fn test_png_modify_already_decoded_image_1() -> Result<()> {
    let image_bytes = include_bytes!("../mia_holding_rust_book.png");

    let cursor = Cursor::new(&image_bytes[..]);
    let image_reader = ImageReader::new(cursor, ImageFormat::Png);

    let backend = ImageRSBackend::from_reader(image_reader)?;
    let decoded_image = backend.decode()?;

    assert_eq!(decoded_image.size, (1920, 1080));
    assert_eq!(decoded_image.colour_type, ImageColourType::Rgba8);

    let mut backend = ImageRSBackend::from_reader(
        ImageReader::new(decoded_image, ImageFormat::Png)
    )?;

    backend.modify(vec![ImageModification::Resize(500, 500)]);
    let decoded_image = backend.decode()?;

    assert_eq!(decoded_image.size, (500, 500));
    assert_eq!(decoded_image.colour_type, ImageColourType::Rgba8);

    save_image::<Rgba<u8>>(decoded_image, "squished_mia.png");

    Ok(())
}

#[test]
fn test_png_modify_already_decoded_image_2() -> Result<()> {
    let image_bytes = include_bytes!("../mia_holding_rust_book.png");

    let cursor = Cursor::new(&image_bytes[..]);
    let image_reader = ImageReader::new(cursor, ImageFormat::Png);

    let backend = ImageRSBackend::from_reader(image_reader)?;
    let decoded_image = backend.decode()?;

    assert_eq!(decoded_image.size, (1920, 1080));
    assert_eq!(decoded_image.colour_type, ImageColourType::Rgba8);

    let mut backend = ImageRSBackend::from_reader(
        ImageReader::new(decoded_image, ImageFormat::Png)
    )?;

    backend.modify(vec![ImageModification::Resize(250, 250)]);
    let decoded_image = backend.decode()?;

    assert_eq!(decoded_image.size, (250, 250));
    assert_eq!(decoded_image.colour_type, ImageColourType::Rgba8);

    save_image::<Rgba<u8>>(decoded_image, "tiny_squished_mia.png");

    Ok(())
}

#[test]
fn test_animated_png_decode_and_modify() -> Result<()> {
    let image_bytes = include_bytes!("../animated_png.png");

    let cursor = Cursor::new(&image_bytes[..]);
    let image_reader = ImageReader::new(cursor, ImageFormat::Png);

    let mut backend = ImageRSBackend::from_reader(image_reader)?;
    backend.modify(vec![ImageModification::Resize(50, 50)]);

    let decoded_image = backend.decode()?;

    assert_eq!(decoded_image.size, (50, 50));
    assert_eq!(decoded_image.colour_type, ImageColourType::Rgba8);

    save_image::<Rgba<u8>>(decoded_image, "tiny_animated_png");

    Ok(())
}

#[test]
fn test_gif_decode_and_modify() -> Result<()> {
    let image_bytes = include_bytes!("../sailor_moon.gif");

    let cursor = Cursor::new(&image_bytes[..]);
    let image_reader = ImageReader::new(cursor, ImageFormat::Gif);

    let mut backend = ImageRSBackend::from_reader(image_reader)?;
    backend.modify(vec![ImageModification::Resize(300, 300)]);

    let decoded_image = backend.decode()?;

    assert_eq!(decoded_image.size, (300, 300));
    assert_eq!(decoded_image.colour_type, ImageColourType::Rgba8);

    save_image::<Rgba<u8>>(decoded_image, "small_and_squished_sailor_moon.gif");

    Ok(())
}

#[test]
fn test_modifying_vertical_png() -> Result<()> {
    let image_bytes = include_bytes!("../example.png");

    let cursor = Cursor::new(&image_bytes[..]);
    let image_reader = ImageReader::new(cursor, ImageFormat::Png);

    let mut backend = ImageRSBackend::from_reader(image_reader)?;
    backend.modify(vec![ImageModification::Resize(138, 154)]);

    let decoded_image = backend.decode()?;

    assert_eq!(decoded_image.size, (138, 154));
    assert_eq!(decoded_image.colour_type, ImageColourType::Rgb8);

    save_image::<Rgb<u8>>(decoded_image, "smaller_example.png");

    Ok(())
}