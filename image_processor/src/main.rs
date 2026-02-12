mod plugin_loader;
mod cli;
mod error;
mod logger;

use std::{fs, io};
use std::io::ErrorKind;
use std::ops::Add;
use std::path::Path;
use clap::Parser;
use image::{ImageReader, RgbaImage};
use plugin_loader::{Plugin};
use crate::cli::{Cli};
use crate::error::ImageProcessorError;

fn main() -> Result<(), ImageProcessorError> {


    let cli = Cli::parse();
    if !Path::new(&cli.plugin).exists(){
        return Err(ImageProcessorError::PathNotExist(io::Error::new(ErrorKind::NotFound, "Path plugin not exists")));
    }
    let plugin_lib = cli.plugin;
    #[cfg(target_os = "windows")]
    let plugin_lib = plugin_lib.add(".dll");
    #[cfg(target_os = "linux")]
    let plugin_lib = plugin_lib.add(".so");
    let plugin_path = Path::new(&cli.plugin_path).join(&plugin_lib);
    if !plugin_path.exists(){
        return Err(ImageProcessorError::PathNotExist(io::Error::new(ErrorKind::NotFound, "Lib plugin not exists")));
    }
    if !Path::new(&cli.input).exists() {
        return Err(ImageProcessorError::PathNotExist(io::Error::new(ErrorKind::NotFound, "Image not exists")));
    }
    if !Path::new(&cli.params).exists() {
        return Err(ImageProcessorError::PathNotExist(io::Error::new(ErrorKind::NotFound, "Params file not exists")));
    }
    let params = fs::read_to_string(cli.params)?;
    let image = ImageReader::open(&cli.input)?.decode()?;
    let mut rgba_img = image.to_rgba8().to_vec();
    {
        let plugin = Plugin::new(&plugin_path.to_str().unwrap())?;
        let plugin = plugin.interface()?;
        (plugin.process_image)(image.width(), image.height(), rgba_img.as_mut_ptr(), params.as_ptr());
    }
    let image = RgbaImage::from_raw(image.width(), image.height(), rgba_img);
    if let Some(image) = image {
        image.save(cli.output.clone())?;
        println!("Image saved to {}", cli.output);
    }
    else { 
        return Err(ImageProcessorError::ConvertFromRawError);
    } 
    println!("Successfully converted!");
    Ok(())
}