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
    vertical_flip: Option<bool>,
    horizontal_flip: Option<bool>,
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
    setup_logger(LevelFilter::Error, &file);
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
        setup_logger(log_level_filter, &file);
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
    if params_config.config.vertical_flip.is_some(){
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
    if params_config.config.horizontal_flip.is_some(){
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

}

