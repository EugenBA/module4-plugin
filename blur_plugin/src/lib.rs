#![warn(missing_docs)]
//! Модуль для реализации обработки обработки изображения - размытия, простой алгоритм
//!
//! Предоставляет функциональность размытию изображения (взатие среднего значения в пределах радиуса размытия)

use std::slice;
use std::ffi::{c_char, c_uint, CStr};
use log::{LevelFilter};
use serde::Deserialize;
use plugins_support::{error::Error, config_parse::ConfigReader};
use plugins_support::logger::{get_log_level, setup_logger};


const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const BYTE_PER_PIXEL: i32 = 4;

#[derive(Deserialize, Debug)]
struct ConfigTransform{
    radius: usize,
    step: usize,
    log_level: Option<String>,
}

/// ```rust
/// /**
///  * Processes an RGBA image buffer by applying configurable transformations such as a blur effect.
///  *
///  * # Safety
///  * This function is marked as `unsafe` due to the following:
///  * - It operates directly on raw pointers (`rgba_data`, `params`) provided by external C code, which requires the caller to ensure proper memory management.
///  * - Improper usage or invalid pointers could result in undefined behavior.
///  *
///  * # Parameters
///  * - `width` (`c_uint`): The width of the image in pixels. Must be greater than 0.
///  * - `height` (`c_uint`): The height of the image in pixels. Must be greater than 0.
///  * - `rgba_data` (`*mut u8`): A mutable raw pointer to the RGBA image buffer. The buffer length must be at least `width * height * 4` bytes.
///  * - `params` (`*const c_char`): A raw pointer to a null-terminated string containing JSON-formatted configuration parameters. Must not be a null pointer.
///  *
///  * # Configuration Parameters (JSON Format)
///  * The `params` parameter must specify a JSON object with the following structure:
///  * - `log_level` (optional, string): The logging level (e.g., "Debug", "Info").
///  * - `radius` (required, integer): The blur radius. Must be greater than 0.
///  * - `step` (required, integer): The number of blur iterations. Must be greater than 0.
///  *
///  * Example `params` JSON:
///  * ```json
///  * {
///  *   "log_level": "Debug",
///  *   "radius": 5,
///  *   "step": 2
///  * }
///  * ```
///  *
///  * # Function Behavior
///  * 1. Initializes a logger for logging messages to a file, named using the package name followed by `.log`.
///  * 2. Validates the input pointers and configuration:
///  *    - Logs an error and returns early if any of the pointers are null or if `width`/`height` is zero.
///  *    - Parses and validates the JSON configuration string.
///  * 3. If valid configuration is provided:
///  *    - Applies a blur effect to the `rgba_data` buffer based on the specified `radius` and `step`.
///  *    - Iterates through the pixel buffer, applying the transformation to each RGBA channel.
///  * 4. Logs the start and completion of the image processing task.
///  *
///  * # Error Handling
///  * - Logs appropriate error messages if:
///  *   - Invalid pointers are provided (`rgba_data` or `params` are null).
///  *   - `width` or `height` is zero.
///  *   - The `radius` or `step` configuration value is missing or invalid (e.g., less than or equal to zero).
///  *   - Issues arise while parsing or converting the configuration string.
///  *
///  * # Important Notes
///  * - This function assumes that the `rgba_data` buffer has a length of at least `width * height * 4` bytes.
///  * - The function modifies the `rgba_data` buffer in place by applying the blur transformation.
///  * - The caller is responsible for ensuring that all raw pointers provided to this function point to valid, properly allocated memory for the duration of the function call.
///  * - The configuration must include valid `radius` and `step` values, as these are prerequisites for applying the image transformations.
///  * - This function is designed to be exposed to C code as an external interface.
///  *
///  * # Example Usage
///  * ```c
///  * // In C code
///  * unsigned int width = 1920;
///  * unsigned int height = 1080;
///  * unsigned char *image_data = ...; // RGBA buffer, allocated elsewhere
///  * const char *config = "{\"radius\": 5, \"step\": 2}";
///  * process_image(width, height, image_data, config);
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
    log::info!("Start converting image");
    let len = (width as usize) * (height as usize) * 4;
    let buf = unsafe { slice::from_raw_parts_mut(rgba_data, len) };
    if params_config.config.radius  > 0 {
        if params_config.config.step > 0 {
            for _ in 0..params_config.config.step {
                for i in 0..width*height{
                    for channel in 0..4 {
                        let result = blur_rgba(buf, i as i32, width as i32, height as i32,
                                               BYTE_PER_PIXEL, params_config.config.radius as i32, channel);
                        if let Some((sum, index)) = result {
                            buf[index] = sum;
                        }
                    }
                }
            }
        }
        else {
            log::error!("Step cannot be 0");
            return;
        }
    }
    else {
        log::error!("Radius cannot be 0");
        return
    }
    log::info!("Finish converting image");
}

/// Размытие RGBA8-изображения box blur'ом.
///
/// # Аргументы
/// * `width`  – ширина в пикселях.
/// * `height` – высота в пикселях.
/// * `buf`    – изменяемый буфер RGBA (длина = width * height * 4).
/// * `radius` – радиус размытия (целое, >= 0). 0 – без изменений.
/// * `passes` – количество проходов (каждый проход: горизонталь + вертикаль).
///
/// # Паника
/// Паникует, если длина buf не равна width * height * 4.
pub fn blur_rgba(buf: &mut [u8], index_pixel: i32, width: i32,
                             height: i32, byte_per_pixel: i32,
                             radius: i32, channel: i32) -> Option<(u8, usize)> {
    assert_eq!(buf.len(), (width as usize * height as usize * 4));
    if radius == 0   {
        log::error!("Radius cannot be 0");
        return None;
    }
    let index = index_pixel*byte_per_pixel + channel;
    let mut count = 0;
    let mut sum = 0.0;
    for i in -radius..=radius {
        let index_column = (i + index_pixel) * byte_per_pixel + channel;
        let row = index /(width * byte_per_pixel);
        let left = row*width*byte_per_pixel + channel ;
        let right = row*width*byte_per_pixel + width*byte_per_pixel + channel;
        if index_column >=left && index_column < right {
            sum += buf[index_column as usize] as f64;
            count += 1;
        }
        let index_row = index + i*width*byte_per_pixel + channel;
        if index_row >= 0 && index_row < buf.len() as i32{
            sum += buf[index_row as usize] as f64;
            count += 1;
        }
    }
    let sum = sum / count as f64;
    Some((sum as u8, index as usize))
}

