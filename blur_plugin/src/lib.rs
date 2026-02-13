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
pub fn box_blur_rgba(width: u32, height: u32, buf: &mut [u8], radius: u32, passes: u32) {
    assert_eq!(buf.len(), (width * height * 4) as usize);
    if radius == 0 || passes == 0 {
        return;
    }
    let w = width as usize;
    let h = height as usize;
    let r = radius as usize;

    // Временный буфер того же размера – для промежуточных результатов
    let mut tmp = vec![0u8; buf.len()];

    // Меняем местами исходный и временный буфер на каждом полупроходе
    let (mut src, mut dst) = (&*buf, &mut tmp[..]);

    for _ in 0..passes {
        // ----- Горизонтальный проход (src → dst) -----
        for y in 0..h {
            let row_start = y * w * 4;
            horizontal_blur_row(&src[row_start..row_start + w * 4], &mut dst[row_start..row_start + w * 4], w, r);
        }

        // Меняем роли: dst становится источником для вертикального прохода
        std::mem::swap(&mut src, &mut dst);

        // ----- Вертикальный проход (src → dst) -----
        // Удобнее обрабатывать столбцы, транспонируя логику: работаем с колонками по x.
        for x in 0..w {
            let col = x * 4;
            vertical_blur_column(src, dst, w, h, col, r);
        }

        // Снова меняем роли для следующего прохода
        std::mem::swap(&mut src, &mut dst);
    }

    // Если после последнего прохода результат оказался во временном буфере – копируем обратно
    if std::ptr::eq(src, &*buf) == false {
        buf.copy_from_slice(src);
    }
}

/// Горизонтальное размытие одной строки (in-place не допускается – src и dst разные).
#[inline]
fn horizontal_blur_row(src_row: &[u8], dst_row: &mut [u8], width: usize, radius: usize) {
    debug_assert_eq!(src_row.len(), width * 4);
    debug_assert_eq!(dst_row.len(), width * 4);

    // Обрабатываем 4 канала независимо
    for c in 0..4 {
        let mut sum = 0u32;
        let mut count = 0usize;

        // Инициализация окна для первого пикселя (x=0)
        for dx in 0..=radius {
            if dx < width {
                sum += src_row[dx * 4 + c] as u32;
                count += 1;
            }
        }

        for x in 0..width {
            // Записываем среднее для текущей позиции
            dst_row[x * 4 + c] = (sum / count as u32) as u8;

            // Убираем пиксель, который уходит слева (x - radius)
            let left = x as isize - radius as isize;
            if left >= 0 {
                sum -= src_row[left as usize * 4 + c] as u32;
                count -= 1;
            }

            // Добавляем пиксель, который появляется справа (x + radius + 1)
            let right = x + radius + 1;
            if right < width {
                sum += src_row[right * 4 + c] as u32;
                count += 1;
            }
        }
    }
}

/// Вертикальное размытие одного столбца.
/// `src` – исходный буфер (высота h), `dst` – целевой буфер.
#[inline]
fn vertical_blur_column(src: &[u8], dst: &mut [u8], width: usize, height: usize, col_start: usize, radius: usize) {
    // col_start – смещение в байтах для первого пикселя столбца (обычно 0,4,8,...)
    for c in 0..4 {
        let mut sum = 0u32;
        let mut count = 0usize;

        // Инициализация окна для первого пикселя (y=0)
        for dy in 0..=radius {
            if dy < height {
                sum += src[(dy * width * 4) + col_start + c] as u32;
                count += 1;
            }
        }

        for y in 0..height {
            dst[(y * width * 4) + col_start + c] = (sum / count as u32) as u8;

            let top = y as isize - radius as isize;
            if top >= 0 {
                sum -= src[(top as usize * width * 4) + col_start + c] as u32;
                count -= 1;
            }

            let bottom = y + radius + 1;
            if bottom < height {
                sum += src[(bottom * width * 4) + col_start + c] as u32;
                count += 1;
            }
        }
    }
}

