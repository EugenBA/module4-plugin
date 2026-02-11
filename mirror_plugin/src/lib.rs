mod error;
mod logger;

use std::ffi::{c_char, c_uint, CStr};
use std::slice;
use log::LevelFilter;
use serde::{Deserialize};
use serde_json;
use crate::error::Error;
use crate::logger::setup_logger;

const PKG_NAME: &str = env!("CARGO_PKG_NAME");

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
    let file = PKG_NAME.to_owned() + ".txt";
    setup_logger(LevelFilter::Error, &file);
    if rgba_data.is_null() || params.is_null() {
        return;
    }
    let len = (width as usize) * (height as usize) * 4;
    let data = unsafe { slice::from_raw_parts_mut(rgba_data, len) };
    let config = unsafe { CStr::from_ptr(params) };
    if let Ok(config) = config.to_str() {} else {
        return;
    }
}

impl TryFrom<&str> for ConfigTransform {
    type Error = Error;
    fn try_from(value: &str) -> Result<Self, Error> {
        Ok(serde_json::from_str(value)?)
    }
}

impl ConfigTransform {
    fn get_log_level(&self) -> LevelFilter {
        match &self.log_level {
            Some(level) => match level.as_str() {
                "error" => LevelFilter::Error,
                "warn" => LevelFilter::Warn,
                "info" => LevelFilter::Info,
                "debug" => LevelFilter::Debug,
                "trace" => LevelFilter::Trace,
                _ => LevelFilter::Error,
            },
            None => LevelFilter::Error,
        }
    }
}

