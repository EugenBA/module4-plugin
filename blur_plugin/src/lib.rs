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
const BYTE_PER_PIXEL: usize = 4;

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
/// # Safety
///  Данная функция  помечена `unsafe`:
///   - Работа напрямую с сырыми указателями (`rgba_data`, `params`) предстаялет external C code
///   - Использования недопустимого указателя или парметров width, height undefined behavior.
/// ```
#[unsafe(no_mangle)]
pub unsafe extern "C" fn process_image(
    width: c_uint,
    height: c_uint,
    rgba_data: *mut u8,
    params: *const c_char,
) {
    let file = PKG_NAME.to_owned() + ".log";
    if let Err(e) = setup_logger(LevelFilter::Debug, &file){
        match e {
            Error::LoggerInitError(_) => {
                log::warn!("Logger init error: {}", e);
            }
            _ => {
                log::error!("Logger init error: {}", e);
                return;
            }
        }
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
    let height: usize = match height.try_into() {
        Ok(h) => h,
        Err(_) => {
            log::error!("Height conversion failed");
            return;
        }
    };
    let width: usize = match width.try_into() {
        Ok(w) => w,
        Err(_) => {
            log::error!("Width conversion failed");
            return;
        }
    };
    let len_image = match width.checked_mul(height) {
        Some(wh) => wh,
        None => {
            log::error!("Length calculation failed");
            return;
        }
    };
    let len_in_pixel = match len_image.checked_mul(BYTE_PER_PIXEL) {
        Some(len) => len,
        None => {
            log::error!("Length calculation failed");
            return;
        }
    };
    let buf = unsafe { slice::from_raw_parts_mut(rgba_data, len_in_pixel) };
    if params_config.config.radius > 0 {
        if params_config.config.step > 0 {
            for _ in 0..params_config.config.step {
                for i in 0..len_image {
                    for channel in 0..4 {
                        let result = blur_rgba(
                            buf,
                            i,
                            width,
                            height,
                            BYTE_PER_PIXEL,
                            params_config.config.radius,
                            channel,
                        );
                        if let Ok((sum, index)) = result {
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
/// * `radius`      – радиус размытия (целое, > 0)
/// * `channel`     – канал (0 - R, 1 - G, 2 - B, 3 - A)
///
pub fn blur_rgba(
    buf: &mut [u8],
    index_pixel: usize,
    width: usize,
    height: usize,
    byte_per_pixel: usize,
    radius: usize,
    channel: usize,
) -> Result<(u8, usize), Error> {
    assert_eq!(buf.len(), width * height * byte_per_pixel);
    if radius == 0 {
        log::error!("Radius cannot be 0");
        return Err(Error::ErrorValue("Radius cannot be 0".to_string()));
    }
    let mut count = 0;
    let mut sum = 0.0;
    let index_pixel: i32 = index_pixel.try_into()?;
    let radius: i32 = radius.try_into()?;
    let channel: i32 = channel.try_into()?;
    let byte_per_pixel: i32 = byte_per_pixel.try_into()?;
    let width: i32 = width.try_into()?;
    let buff_len: i32 = buf.len().try_into()?;
    let index = channel
        + match index_pixel.checked_mul(byte_per_pixel) {
            Some(index) => index,
            None => {
                log::error!("Overflow type index");
                return Err(Error::OverflowError);
            }
        };
    for i in -radius..=radius {
        let index_column = channel
            + match (i + index_pixel).checked_mul(byte_per_pixel) {
                Some(index) => index,
                None => {
                    log::error!("Overflow type index_column");
                    return Err(Error::OverflowError);
                }
            };
        let len_width_in_byte = match width.checked_mul(byte_per_pixel) {
            Some(len) => len,
            None => {
                log::error!("Overflow type len_width_in_byte");
                return Err(Error::OverflowError);
            }
        };
        let row = match index.checked_div(len_width_in_byte) {
            Some(row) => row,
            None => {
                return {
                    log::error!("Overflow type row");
                    Err(Error::OverflowError)
                };
            }
        };
        let left_base_index = match row.checked_mul(len_width_in_byte) {
            Some(left_base_index) => left_base_index,
            None => {
                log::error!("Overflow type left_base_index");
                return Err(Error::OverflowError);
            }
        };
        let left = left_base_index + channel;
        let right = left_base_index + len_width_in_byte + channel;
        if index_column >= left
            && index_column < right
            && left >= 0
            && right < buff_len
            && left < right
        {
            sum += buf[index_column as usize] as f64;
            count += 1;
        }
        let index_row = channel
            + index
            + match i.checked_mul(len_width_in_byte) {
                Some(index) => index,
                None => {
                    log::error!("Overflow type index_row");
                    return Err(Error::OverflowError);
                }
            };
        if index_row >= 0 && index_row < buff_len {
            sum += buf[index_row as usize] as f64;
            count += 1;
        }
    }
    let sum = sum / count as f64;
    Ok((sum as u8, index as usize))
}

#[cfg(test)]
mod tests {
    use std::ffi::CString;
    use super::*;
    #[test]
    fn test_blur_rgba() {
        let mut buf = vec![1; 400];
        let result = blur_rgba(&mut buf, 0, 10, 10, 4, 1, 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0, 1);
    }
    #[test]
    fn test_blur_image() {
        let mut buf = (0..16).collect::<Vec<_>>();
        let json = r#"{"step": 1, "radius": 2}"#;
        let params_cstring = CString::new(json).unwrap();
        unsafe { process_image(2, 2, buf.as_mut_ptr(), params_cstring.as_ptr()) };
        assert_eq!(buf, vec![3, 4, 6, 7, 5, 7, 5, 7, 5, 8, 8, 9, 8, 9, 4, 7]);
    }
    /// Тест радиус i32:MAX для теста переполнения
    /// Так как для теста перполнения размера изображения
    /// несобходим буффер размера i32:MAX*1*4 ~530Mp
    /// тест сделаем на перполнение радиуса установив его в i32:MAX
    #[test]
    fn test_blur_rgba_overflow(){
        let mut buf = (0..16).collect::<Vec<_>>();
        let radius  = i32::MAX as usize;
        let result = blur_rgba(&mut buf, 0, 2,
                               2, 4, radius, 0);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::OverflowError))
    }

}
