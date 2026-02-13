use std::slice;
use std::ffi::{c_char, c_uint, CStr};
use std::{ptr};
use log::{LevelFilter};
use serde::Deserialize;
use plugins_support::{error::Error, config_parse::ConfigReader};
use plugins_support::logger::{get_log_level, setup_logger};


const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const BYTE_PER_PIXEL: usize = 4;

#[derive(Deserialize, Debug)]
struct ConfigTransform{
    radius: Option<usize>,
    step: Option<usize>,
    log_level: Option<String>,
}
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
    let len = (width as usize) * (height as usize) * 4;
    let buf = unsafe { slice::from_raw_parts_mut(rgba_data, len) };
    if let Some(radius) = params_config.config.radius && radius > 0{
        if let Some(step) = params_config.config.step && step > 0 {
            for _ in 0..step {
                for i in 0..len {
                    for channel in 0..4 {
                        let result = blur_rgba(buf, i, width as usize, height as usize,
                                               BYTE_PER_PIXEL, radius as i32, channel);
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
pub fn blur_rgba(buf: &mut [u8], index_pixel: usize, width: usize,
                             height: usize, byte_per_pixel: usize,
                             radius: i32, channel: u32) -> Option<(u8, usize)> {
    assert_eq!(buf.len(), (width * height * 4));
    if radius == 0   {
        return None;
    }
    let index = index_pixel*byte_per_pixel + channel as usize;
    if index >= buf.len() {
        return None;
    }
    let mut count = 0;
    let mut sum = 0;
    for i in -radius..=radius {
        let index_column = (i as usize + index_pixel) * byte_per_pixel + channel as usize;
        let row = index /(width * byte_per_pixel);
        let left = row*width*byte_per_pixel;
        let right = row*width*byte_per_pixel + width*byte_per_pixel;
        if index_column >=left && index_column < right {
            sum += buf[index_column];
            count += 1;
        }
        let index_row = index + i as usize*width*byte_per_pixel + channel as usize;
        if index_row < buf.len() {
            sum += buf[index_row];
            count += 1;
        }
    }
    return Some((sum / count, index));

}

