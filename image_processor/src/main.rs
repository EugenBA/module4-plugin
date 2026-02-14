//! Модуль для реализации обработки изображений с плагинами
//!
//! Предоставляет функциональность по обработке изображений с подключаемыми плагинами

#![warn(missing_docs)]
mod cli;
mod error;
mod logger;
mod plugin_loader;

use crate::cli::Cli;
use crate::error::ImageProcessorError;
use crate::logger::{get_logger_filter, setup_logger};
use clap::Parser;
use image::{ImageReader, RgbaImage};
use plugin_loader::Plugin;
use std::ffi::CString;
use std::io::ErrorKind;
use std::ops::Add;
use std::path::Path;
use std::{fs, io};

const PKG_NAME: &str = env!("CARGO_PKG_NAME");

fn main() -> Result<(), ImageProcessorError> {
    let cli = Cli::parse();
    let file = PKG_NAME.to_owned() + ".log";
    let log_level_filter = get_logger_filter(&cli.log_level);
    setup_logger(log_level_filter, &file);
    log::info!("Starting image processor");
    if !Path::new(&cli.plugin_path).is_dir() {
        log::error!("Could not find plugin {}", cli.plugin_path);
        return Err(ImageProcessorError::PathNotExist(io::Error::new(
            ErrorKind::NotFound,
            "Path plugin not exists",
        )));
    }
    let plugin_lib = cli.plugin;
    #[cfg(target_os = "windows")]
    let plugin_lib = plugin_lib.add(".dll");
    #[cfg(target_os = "linux")]
    let plugin_lib = plugin_lib.add(".so");
    let plugin_path = Path::new(&cli.plugin_path).join(&plugin_lib);
    log::info!("Plugin: {}", plugin_path.display());
    if !plugin_path.exists() {
        log::error!("Could not find plugin path {}", plugin_path.display());
        return Err(ImageProcessorError::PathNotExist(io::Error::new(
            ErrorKind::NotFound,
            "Lib plugin not exists",
        )));
    }
    if !Path::new(&cli.input).exists() {
        log::error!("Could not find image {}", cli.input);
        return Err(ImageProcessorError::PathNotExist(io::Error::new(
            ErrorKind::NotFound,
            "Image not exists",
        )));
    }
    if !Path::new(&cli.params).exists() {
        log::error!("Could not find params file {}", cli.params);
        return Err(ImageProcessorError::PathNotExist(io::Error::new(
            ErrorKind::NotFound,
            "Params file not exists",
        )));
    }
    log::info!("Image input: {}", cli.input);
    log::info!(
        "Image plugin lib: {}",
        plugin_path.to_str().unwrap_or("unknown")
    );
    let params = fs::read_to_string(cli.params)?;
    let image = ImageReader::open(&cli.input)?.decode()?;
    let mut rgba_img = image.to_rgba8().to_vec();
    {
        let plugin = Plugin::new(&plugin_path.to_str().unwrap())?;
        let plugin = plugin.interface()?;
        let params_cstring = CString::new(params)?;
        (plugin.process_image)(
            image.width(),
            image.height(),
            rgba_img.as_mut_ptr(),
            params_cstring.as_ptr(),
        );
    }
    let image = RgbaImage::from_raw(image.width(), image.height(), rgba_img);
    if let Some(image) = image {
        image.save(cli.output.clone())?;
        println!("Image saved to {}", cli.output);
        log::info!("Image successfully saved to {}", cli.output);
    } else {
        log::error!("Error convert image");
        return Err(ImageProcessorError::ConvertFromRawError);
    }
    println!("Successfully converted!");
    Ok(())
}
