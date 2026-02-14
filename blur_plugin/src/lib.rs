#![warn(missing_docs)]
//! Модуль для реализации обработки обработки изображения - размытия, простой алгоритм
//!
//! Предоставляет функциональность размытию изображения (взатие среднего значения в пределах радиуса размытия)

use log::LevelFilter;
use plugins_support::logger::{get_log_level, setup_logger};
use plugins_support::{config_parse::ConfigReader, error::Error};
use serde::Deserialize;
use std::ffi::{CStr, c_char, c_uint};
use std::slice;

const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const BYTE_PER_PIXEL: i32 = 4;

#[derive(Deserialize, Debug)]
struct ConfigTransform {
    radius: usize,
    step: usize,
    log_level: Option<String>,
}

/// ```rust
///
///  Трансформация RGBA буффера эффектома размытия
///
///  Safety
///  Данная функция  помечена `unsafe`:
///   - Работа напрямую с сырыми указателями (`rgba_data`, `params`) предстаялет external C code
///   - Использования недопустимого указателя или парметров width, height undefined behavior.
///
///  # Параметры
///   - `width` (`c_uint`):  ширина изображения в пикселях
///   - `height` (`c_uint`): высота изображения в пикселях
///   - `rgba_data` (`*mut u8`): Мутабельный сырой указатель на RGBA буфер. Размер буфера `width * height * 4` bytes.
///   - `params` (`*const c_char`): Сырой указатель на нуль терменированноу строку  конфигурации JSON-формата.
///
///  # Параметры конфигурации (JSON формат)
///   - `log_level` (optional, string): уровень логирования ("Debug", "Info").
///   - `radius` (required, integer): Радиус сглаживания. Должен быть больше 0
///   - `step` (required, integer): Количество итераций. Должен быть больше 0
///
///  # Пример JSON:
///  ```json
///  {
///     "log_level": "Debug",
///     "radius": 5,
///     "step": 2
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
/// ```
#[unsafe(no_mangle)]
pub unsafe extern "C" fn process_image(
    width: c_uint,
    height: c_uint,
    rgba_data: *mut u8,
    params: *const c_char,
) {
    let file = PKG_NAME.to_owned() + ".log";
    if let Err(_) = setup_logger(LevelFilter::Debug, &file){
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
    log::info!("Start converting image");
    let len = (width as usize) * (height as usize) * 4;
    let buf = unsafe { slice::from_raw_parts_mut(rgba_data, len) };
    if params_config.config.radius > 0 {
        if params_config.config.step > 0 {
            for _ in 0..params_config.config.step {
                for i in 0..width * height {
                    for channel in 0..4 {
                        let result = blur_rgba(
                            buf,
                            i as i32,
                            width as i32,
                            height as i32,
                            BYTE_PER_PIXEL,
                            params_config.config.radius as i32,
                            channel,
                        );
                        if let Some((sum, index)) = result {
                            buf[index] = sum;
                        }
                    }
                }
            }
        } else {
            log::error!("Step cannot be 0");
            return;
        }
    } else {
        log::error!("Radius cannot be 0");
        return;
    }
    log::info!("Finish converting image");
}

/// Размытие RGBA8-изображения box blur'ом.
///
/// # Аргументы
/// * `buf`         – изменяемый буфер RGBA (длина = width * height * 4).
/// * `index_pixel` - текущий индекс пикселя
/// * `width`       – ширина в пикселях.
/// * `height`      – высота в пикселях.
/// * `byte_per_pixel` – количество байт на пиксель
/// * `radius`      – радиус размытия (целое, >= 0). 0 – без изменений.
/// * `channel`     – канал (0 - R, 1 - G, 2 - B, 3 - A)
///
/// # Паника
/// Паникует, если длина buf не равна width * height * 4.
pub fn blur_rgba(
    buf: &mut [u8],
    index_pixel: i32,
    width: i32,
    height: i32,
    byte_per_pixel: i32,
    radius: i32,
    channel: i32,
) -> Option<(u8, usize)> {
    assert_eq!(buf.len(), (width as usize * height as usize * 4));
    if radius == 0 {
        log::error!("Radius cannot be 0");
        return None;
    }
    let index = index_pixel * byte_per_pixel + channel;
    let mut count = 0;
    let mut sum = 0.0;
    for i in -radius..=radius {
        let index_column = (i + index_pixel) * byte_per_pixel + channel;
        let row = index / (width * byte_per_pixel);
        let left = row * width * byte_per_pixel + channel;
        let right = row * width * byte_per_pixel + width * byte_per_pixel + channel;
        if index_column >= left && index_column < right {
            sum += buf[index_column as usize] as f64;
            count += 1;
        }
        let index_row = index + i * width * byte_per_pixel + channel;
        if index_row >= 0 && index_row < buf.len() as i32 {
            sum += buf[index_row as usize] as f64;
            count += 1;
        }
    }
    let sum = sum / count as f64;
    Some((sum as u8, index as usize))
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_blur_rgba() {
        let mut buf = vec![1; 400];
        let result = blur_rgba(&mut buf, 0, 10, 10, 4, 1, 0);
        assert!(result.is_some());
        assert_eq!(result.unwrap().0, 1);
    }
}