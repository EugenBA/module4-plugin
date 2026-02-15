//! Модуль для реализации логирования
//!
//! Предоставляет функциональность поддержки логирования

use env_logger::{Builder, Target};
use log::LevelFilter;
use std::fs::File;
use std::io::Write;
use crate::error::Error;

///
/// Инициализация логера
///
///
/// # Параметры
///
/// * `level` - LevelFilter уровень логирования ("Error", "Warn", "Info", "Debug", "Trace").
/// * `file` - Файл для записи логов.
///
/// # Формат лога
///
/// Лог выводится
///
/// ```txt
/// YYYY-MM-DD HH:MM:SS.mmm [LOG_LEVEL] file_name:line_number - сообщение
/// ```
///
/// Example:
///
/// ```txt
/// 2023-03-01 12:34:56.789 [INFO] main.rs:42 - Application started successfully
/// ```
///
/// # Usage
///
///```ignore
/// use log::LevelFilter;
///
/// setup_logger(LevelFilter::Info, "app.log");
/// log::info!("This is an informational message.");
///```
pub fn setup_logger(level: LevelFilter, file: &str) -> Result<(), Error> {
    let log_file = File::create(file)?;
    Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] {}:{} - {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level(),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.args()
            )
        })
        .target(Target::Pipe(Box::new(log_file)))
        .filter(None, level) // Уровень по умолчанию
        .write_style(env_logger::WriteStyle::Always) // Всегда использовать цвета
        .init();
    Ok(())
}

///
/// Преобразование теста уровня логирования в LogFilter
///
/// # Параметры
///
/// * `config_str` - строка с уровнем логирования
/// *  - `"error"`: Translated to `LevelFilter::Error`.
/// *  - `"warn"`: Translated to `LevelFilter::Warn`.
/// *  - `"info"`: Translated to `LevelFilter::Info`.
/// *  - `"debug"`: Translated to `LevelFilter::Debug`.
/// *  - `"trace"`: Translated to `LevelFilter::Trace`.
///
/// # Возращает
/// LevelFilter
///
/// # Пример
///
///```ignore
/// use log::LevelFilter;
///
/// let level = get_log_level("info");
/// assert_eq!(level, LevelFilter::Info);
///
/// let level = get_log_level("unknown");
/// assert_eq!(level, LevelFilter::Error); // Defaults to Error
/// ```
///
pub fn get_log_level(config_str: &str) -> LevelFilter {
    match config_str {
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        _ => LevelFilter::Error,
    }
}
