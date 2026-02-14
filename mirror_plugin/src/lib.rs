//! Модуль для реализации обработки изображения - зеркальное отражение
//!
//! Предоставляет функциональность по транформации изображения - вертикальное, горизотальное отражение
#![warn(missing_docs)]
use std::ffi::{c_char, c_uint, CStr};
use std::{ptr};
use log::{log, LevelFilter};
use serde::Deserialize;
use plugins_support::{error::Error, config_parse::ConfigReader};
use plugins_support::logger::{get_log_level, setup_logger};


const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const BYTE_PER_PIXEL: usize = 4;

#[derive(Deserialize, Debug)]
struct ConfigTransform{
    vertical_flip: Option<bool>,
    horizontal_flip: Option<bool>,
    log_level: Option<String>,
}

/// ```rust
/// /**
///  * Processes an image buffer by applying transformations such as vertical flip or horizontal flip based on the provided configuration.
///  *
///  * # Parameters
///  * - `width` (`c_uint`): The width of the image in pixels. Must be greater than 0.
///  * - `height` (`c_uint`): The height of the image in pixels. Must be greater than 0.
///  * - `rgba_data` (`*mut u8`): A mutable pointer to the RGBA pixel data of the image. Must not be null.
///  * - `params` (`*const c_char`): A pointer to a string containing a JSON configuration for the plugin. Must not be null.
///  *
///  * # Configuration (`params`)
///  * The `params` parameter should contain a JSON configuration string that includes the following optional fields:
///  * - `log_level` (string): The desired logging level (e.g., "debug", "info", "error").
///  * - `vertical_flip` (bool): Whether to vertically flip the image. Defaults to `false`.
///  * - `horizontal_flip` (bool): Whether to horizontally flip the image. Defaults to `false`.
///  *
///  * # Behavior
///  * 1. If `params` is null or cannot be parsed, an error is logged, and the function returns without processing the image.
///  * 2. If `rgba_data` is null, or if `width` or `height` is zero, an error is logged, and the function returns without processing the image.
///  * 3. If the `vertical_flip` option in the configuration is set to `true`, the image is flipped vertically.
///  * 4. If the `horizontal_flip` option in the configuration is set to `true`, the image is flipped horizontally.
///  *
///  * # Logging
///  * - A log file is created with the name `<PKG_NAME>.log`, where `<PKG_NAME>` is a predefined constant.
///  * - Logs are written at various levels (`info`, `debug`, `error`) to help track the state of the function.
///  *
///  * # Safety
///  * This function uses unsafe code to manipulate raw pointers:
///  * - Make sure that the `rgba_data` pointer points to a valid and properly sized memory region.
///  * - Ensure the image dimensions (`width` and `height`) match the size of the `rgba_data` buffer.
///  *
///  * # Errors Logged
///  * - "Pointer params is_null": `params` is null.
///  * - "Invalid config file": The `params` JSON configuration is invalid or cannot be parsed.
///  * - "Null pointer rgba_data": `rgba_data` is null.
///  * - "width cannot be 0": `width` is 0.
///  * - "height cannot be 0": `height` is 0.
///  * - "Width out of bounds": The computed row size for vertical flipping exceeds the bounds of `usize`.
///  * - "Height out of bounds": The computed column size for horizontal flipping exceeds the bounds of `usize`.
///  *
///  * # Example Usage
///  * ```rust
///  * use std::os::raw::{c_char, c_uint};
///  *
///  * let width: c_uint = 800;
///  * let height: c_uint = 600;
///  * let mut rgba_data: Vec<u8> = vec![255; (width * height * 4) as usize]; // Placeholder image data.
///  * let config = r#"{
///  *     "log_level": "info",
///  *     "vertical_flip": true,
///  *     "horizontal_flip": false
///  * }"#;
///  *
///  * unsafe {
///  *     process_image(
///  *         width,
///  *         height,
///  *         rgba_data.as_mut_ptr(),
///  *         config.as_ptr() as *const c_char
///  *     );
///  * }
///  * ```
///  */
/// ```
#[unsafe(no_mangle)]
pub extern "C" fn process_image(
                                width: c_uint,
                                height: c_uint,
                                rgba_data: *mut u8,
                                params: *const c_char
                                ) {
    let file = PKG_NAME.to_owned() + ".log";
    setup_logger(LevelFilter::Debug, &file);
    log::info!("Start plugin {}", &file);
    if params.is_null(){
         log::error!("Pointer params is_null");
        return;
    }
    let config = unsafe { CStr::from_ptr(params) };
    let params_config = match config.to_str() {
        Ok(config) =>
            {
                let config: Result<ConfigReader<ConfigTransform>, Error> = ConfigReader::try_from(config);
                if let Ok(config) = config{
                    config
                }
                else {
                    log::error!("Error converting config to string");
                    return;
                }
            },
        _ => {
            log::error!("Invalid config file");
            return;
        }
    };
    if let Some(log_level) = params_config.config.log_level
    {
        let log_level_filter = get_log_level(&log_level);
        //setup_logger(log_level_filter, &file);
    }
    if rgba_data.is_null() {
        log::error!("Null pointer rgba_data");
        return;
    }
    if width == 0{
        log::error!("width cannot be 0");
        return;
    }
    if height == 0{
        log::error!("height cannot be 0");
        return;
    }
    if let Some(vertical) = params_config.config.vertical_flip && vertical{
        log::info!("Flipped vertical");
        let row_size = match (width as usize).checked_mul(BYTE_PER_PIXEL) {
            Some(size) => size,
            None => {
                log::error!("Width out of bounds");
                return
            },
        };
        let height = height as usize;
        for i in 0..(height / 2) {
            let top_offset = i * row_size;
            let bottom_offset = (height - 1 - i) * row_size;
            unsafe {
                let top_ptr = rgba_data.add(top_offset);
                let bottom_ptr = rgba_data.add(bottom_offset);
                ptr::swap_nonoverlapping(top_ptr, bottom_ptr, row_size);
            }
        }

    }
    if let Some(horizontal) = params_config.config.horizontal_flip && horizontal{
        log::info!("Flipped horizontal");
        let col_size = match (height as usize).checked_mul(BYTE_PER_PIXEL) {
            Some(size) => size,
            None => {
                log::error!("Height out of bounds");
                return
            },
        };
        let width = width as usize;
        for i in 0..(width / 2) {
            let left_offset = i * col_size;
            let right_offset = (width - 1 - i) * col_size;
            unsafe {
                let left_ptr = rgba_data.add(left_offset);
                let right_ptr = rgba_data.add(right_offset);
                ptr::swap_nonoverlapping(left_ptr, right_ptr, col_size);
            }
        }

    }
    log::info!("Image processed successfully");
}

