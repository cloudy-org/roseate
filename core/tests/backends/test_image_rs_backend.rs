use std::{fs, io::Cursor};

use roseate_core::{self, backends::{backend::DecodeBackend, image_rs::ImageRSBackend}, error::Result, image::{ImageColourType}, modifications::ImageModification, reader::{ImageFormat, ImageReader}};

use crate::backends::{IMAGE_DUMP_PATH, save_image};

#[test]
fn init() {
    env_logger::init();
    let _ = fs::create_dir(IMAGE_DUMP_PATH);
}

#[test]
fn test_decode() -> Result<()> {
    let image_bytes = include_bytes!("../mov_cli_logo.png");

    let cursor = Cursor::new(&image_bytes[..]);
    let image_reader = ImageReader::new(cursor, ImageFormat::Png);

    let backend = ImageRSBackend::from_reader(image_reader)?;
    let decoded_image = backend.decode()?;

    assert_eq!(decoded_image.size, (750, 250));
    assert_eq!(decoded_image.colour_type, ImageColourType::Rgba8);

    save_image(decoded_image, "mov-cli.png");

    Ok(())
}

#[test]
fn test_modify_and_decode() -> Result<()> {
    let image_bytes = include_bytes!("../Mia_Sakurajima_Holding_Rust_Book.png");

    let cursor = Cursor::new(&image_bytes[..]);
    let image_reader = ImageReader::new(cursor, ImageFormat::Png);

    let mut backend = ImageRSBackend::from_reader(image_reader)?;
    backend.modify(vec![ImageModification::Resize(1280, 720)]);

    let decoded_image = backend.decode()?;

    assert_eq!(decoded_image.size, (1280, 720));
    assert_eq!(decoded_image.colour_type, ImageColourType::Rgba8);

    save_image(decoded_image, "smaller_mia.png");

    Ok(())
}

#[test]
fn test_modify_already_decoded_image() -> Result<()> {
    let image_bytes = include_bytes!("../Mia_Sakurajima_Holding_Rust_Book.png");

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

    save_image(decoded_image, "squished_mia.png");

    Ok(())
}