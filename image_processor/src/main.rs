//! Модуль для реализации обработки изображений с плагинами
//!
//! Предоставляет функциональность по обработке изображений с подключаемыми плагинами

#![warn(missing_docs)]
mod cli;
mod error;
mod plugin_loader;

use crate::cli::Cli;
use crate::error::ImageProcessorError;
use clap::Parser;
use image::{ImageReader, RgbaImage};
use plugin_loader::Plugin;
use std::ffi::CString;
use std::io::ErrorKind;
use std::ops::Add;
use std::{fs, io};
use plugins_support::logger::{get_log_level, setup_logger};

const PKG_NAME: &str = env!("CARGO_PKG_NAME");

fn main() -> Result<(), ImageProcessorError> {
    let cli = Cli::parse();
    let file = PKG_NAME.to_owned() + ".log";
    let log_level_filter = get_log_level(&cli.log_level);
    if let  Err(_) = setup_logger(log_level_filter, &file)
    {
        return Err(ImageProcessorError::LoggerSetupFailed);
    }
    log::info!("Starting image processor");
    if !cli.plugin_path.exists() {
        log::error!("Could not find plugin {}", cli.plugin_path.to_string_lossy());
        return Err(ImageProcessorError::PathNotExist(io::Error::new(
            ErrorKind::NotFound,
            "Path plugin not exists",
        )));
    }
    let plugin_lib = cli.plugin.to_str().unwrap().to_owned();
    #[cfg(target_os = "windows")]
    let plugin_lib = plugin_lib.add(".dll");
    #[cfg(target_os = "linux")]
    let plugin_lib = plugin_lib.add(".so");
    let plugin_path = cli.plugin_path.join(&plugin_lib);
    log::info!("Plugin: {}", plugin_path.display());
    if !plugin_path.exists() {
        log::error!("Could not find plugin path {}", plugin_path.display());
        return Err(ImageProcessorError::PathNotExist(io::Error::new(
            ErrorKind::NotFound,
            "Lib plugin not exists",
        )));
    }
    if !cli.input.exists() {
        log::error!("Could not find image {}", cli.input.to_string_lossy());
        return Err(ImageProcessorError::PathNotExist(io::Error::new(
            ErrorKind::NotFound,
            "Image not exists",
        )));
    }
    if !cli.params.exists() {
        log::error!("Could not find params file {}", cli.params.to_string_lossy());
        return Err(ImageProcessorError::PathNotExist(io::Error::new(
            ErrorKind::NotFound,
            "Params file not exists",
        )));
    }
    log::info!("Image input: {}", cli.input.to_string_lossy());
    log::info!(
        "Image plugin lib: {}",
        plugin_path.to_str().unwrap_or("unknown")
    );
    let params = fs::read_to_string(cli.params)?;
    let image = ImageReader::open(&cli.input)?.decode()?;
    let mut rgba_img = image.to_rgba8().to_vec();
    let plugin = Plugin::new(&plugin_path.to_str().unwrap())?;
    let plugin = plugin.interface()?;
    let params_cstring = CString::new(params)?;
    unsafe {
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
        println!("Image saved to {}", cli.output.to_string_lossy());
        log::info!("Image successfully saved to {}", cli.output.to_string_lossy());
    } else {
        log::error!("Error convert image");
        return Err(ImageProcessorError::ConvertFromRawError);
    }
    println!("Successfully converted!");
    Ok(())
}
