//! Модуль для реализации обработки изображения - зеркальное отражение
//!
//! Предоставляет функциональность по транформации изображения - вертикальное, горизотальное отражение
#![warn(missing_docs)]
use log::LevelFilter;
use plugins_support::logger::{get_log_level, setup_logger};
use plugins_support::{config_parse::ConfigReader, error::Error};
use serde::Deserialize;
use std::ffi::{CStr, c_char, c_uint};
use std::ptr;

const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const BYTE_PER_PIXEL: usize = 4;

#[derive(Deserialize, Debug)]
struct ConfigTransform {
    vertical_flip: Option<bool>,
    horizontal_flip: Option<bool>,
    log_level: Option<String>,
}

/// ```rust
///
///  Трансформация RGBA буффера горизонтальное и/или вертикальное отражение
///
///
///  # Параметры
///   - `width` (`c_uint`):  ширина изображения в пикселях
///   - `height` (`c_uint`): высота изображения в пикселях
///   - `rgba_data` (`*mut u8`): Мутабельный сырой указатель на RGBA буфер. Размер буфера `width * height * 4` bytes.
///   - `params` (`*const c_char`): Сырой указатель на нуль терменированноу строку  конфигурации JSON-формата.
///
///  # Параметры конфигурации (JSON формат)
///   - `log_level` (optional, string): уровень логирования ("Debug", "Info").
///   - `vertical_flip` (optional, bool): Вертикальное отражение
///   - `horizontal_flip` (optional, bool): Горизонтальное отражение
///
///  # Пример JSON:
///  ```json
///  {
///     "log_level": "Debug",
///     "vertical_flip": true,
///     "horizontal_flip": true
///  }
///  ```
///
///
///   # Error Handling
///   - Ошибки фиксируются в логе
///
///
///   # Пример С
///  ```c
///  // In C code
///   unsigned int width = 1920;
///   unsigned int height = 1080;
///   unsigned char *image_data = ...; // RGBA buffer, allocated elsewhere
///   const char *config = "{\"radius\": 5, \"step\": 2}";
///   process_image(width, height, image_data, config);
///   ```
///
/// # Safety
/// Данная функция  помечена `unsafe`:
///  - Работа напрямую с сырыми указателями (`rgba_data`, `params`) предстаялет external C code
///  - Использования недопустимого указателя или парметров width, height undefined behavior.
///```
#[unsafe(no_mangle)]
pub unsafe extern "C" fn process_image(
    width: c_uint,
    height: c_uint,
    rgba_data: *mut u8,
    params: *const c_char,
) {
    let file = PKG_NAME.to_owned() + ".log";
    if setup_logger(LevelFilter::Debug, &file).is_err(){
        return;
    }
    log::info!("Start plugin {}", &file);
    if params.is_null() {
        log::error!("Pointer params is_null");
        return;
    }
    let config = unsafe { CStr::from_ptr(params) };
    let params_config = match config.to_str() {
        Ok(config) => {
            let config: Result<ConfigReader<ConfigTransform>, Error> =
                ConfigReader::try_from(config);
            if let Ok(config) = config {
                config
            } else {
                log::error!("Error converting config to string");
                return;
            }
        }
        _ => {
            log::error!("Invalid config file");
            return;
        }
    };
    if let Some(log_level) = params_config.config.log_level {
        let log_level_filter = get_log_level(&log_level);
        log::set_max_level(log_level_filter);
    }
    if rgba_data.is_null() {
        log::error!("Null pointer rgba_data");
        return;
    }
    if width == 0 {
        log::error!("width cannot be 0");
        return;
    }
    if height == 0 {
        log::error!("height cannot be 0");
        return;
    }
    let width: usize = match  width.try_into(){
        Ok(w) => w,
        Err(_) => {
            log::error!("Width conversion failed");
            return;
        }
    };
    let height: usize = match height.try_into(){
       Ok(h) => h,
        Err(_) => {
            log::error!("Height conversion failed");
            return;
        }
    };
    if let Some(vertical) = params_config.config.vertical_flip
        && vertical
    {
        log::info!("Flipped vertical");
        let row_size = match width.checked_mul(BYTE_PER_PIXEL) {
            Some(size) => size,
            None => {
                log::error!("Width out of bounds");
                return;
            }
        };
        for i in 0..(height / 2) {
            let top_offset =  match i.checked_mul(row_size)
            {
                Some(top_offset) => top_offset,
                None => {
                    log::error!("Top offset out of bounds");
                    return;
                }
            };
            let bottom_offset = match (height - 1 - i).checked_mul(row_size){
                Some(bottom_offset) => bottom_offset,
                None => {
                    log::error!("Bottom offset out of bounds");
                    return;
                }
            };
            unsafe {
                let top_ptr = rgba_data.add(top_offset);
                let bottom_ptr = rgba_data.add(bottom_offset);
                ptr::swap_nonoverlapping(top_ptr, bottom_ptr, row_size);
            }
        }
    }
    if let Some(horizontal) = params_config.config.horizontal_flip
        && horizontal
    {
        log::info!("Flipped horizontal");
        let row_size = match width.checked_mul(BYTE_PER_PIXEL)
        {
            Some(row_size) => row_size,
            None => {
                log::error!("Row size out of bounds");
                return;
            }
        };

        for y in 0..height {
            let row_start = y * row_size;
            for x in 0..width / 2 {
                let left_offset = row_start + match x.checked_mul(BYTE_PER_PIXEL){
                    Some(left_offset) => left_offset,
                    None => {
                        log::error!("Left offset out of bounds");
                        return;
                    }
                };
                let right_offset = row_start + match (width - 1 - x).checked_mul(BYTE_PER_PIXEL){
                    Some(right_offset) => right_offset,
                    None => {
                        log::error!("Right offset out of bounds");
                        return;
                    }
                };
                unsafe {
                    let left_ptr = rgba_data.add(left_offset);
                    let right_ptr = rgba_data.add(right_offset);
                    // Обмениваем 4 байта (целый пиксель)
                    ptr::swap_nonoverlapping(left_ptr, right_ptr, BYTE_PER_PIXEL);
                }
            }
        }
    }
    log::info!("Image processed successfully");
}

#[cfg(test)]
mod tests {
    use std::ffi::CString;
    use super::*;
    #[test]
    fn test_mirror_image_vertical() {
        let mut buf = (0..16).collect::<Vec<_>>();
        let json = r#"{"vertical_flip": true, "horizontal_flip": false}"#;
        let params_cstring = CString::new(json).unwrap();
        unsafe { process_image(2, 2, buf.as_mut_ptr(), params_cstring.as_ptr()) };
        assert_eq!(buf, vec![8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7]);
    }

    #[test]
    fn test_mirror_image_horizontal() {
        let mut buf = (0..16).collect::<Vec<_>>();
        let json = r#"{"vertical_flip": false, "horizontal_flip": true}"#;
        let params_cstring = CString::new(json).unwrap();
        unsafe { process_image(2, 2, buf.as_mut_ptr(), params_cstring.as_ptr()) };
        assert_eq!(buf, vec![4, 5, 6, 7, 0, 1, 2, 3, 12, 13, 14, 15, 8, 9, 10, 11]);
    }
}
