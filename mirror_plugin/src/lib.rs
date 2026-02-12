use std::ffi::{c_char, c_uint, CStr};
use std::slice;
use log::{LevelFilter};
use serde::Deserialize;
use serde_json;
use plugins_support::error::Error;
use plugins_support::logger::{get_log_level, setup_logger};


const PKG_NAME: &str = env!("CARGO_PKG_NAME");

#[derive(Deserialize, Debug)]
struct ConfigTransform{
    vertical_flip: Option<bool>,
    horizontal_flip: Option<bool>,
    log_level: Option<String>,
}

impl TryFrom<&str> for ConfigTransform {
    type Error = Error;
    fn try_from(value: &str) -> Result<Self, Error> {
        Ok(serde_json::from_str(value)?)
    }
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
                let config = ConfigTransform::try_from(config);
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
    if let Some(log_level) = params_config.log_level
    {
        let log_level_filter = get_log_level(&log_level);
        setup_logger(log_level_filter, &file);
    }
    if rgba_data.is_null() {
        log::error!("Null pointer rgba_data");
        return;
    }
    let len = (width as usize) * (height as usize) * 4;
    let data = unsafe { slice::from_raw_parts_mut(rgba_data, len) };
    if params_config.vertical_flip.is_some(){

    }
    if params_config.horizontal_flip.is_some(){

    }

}

