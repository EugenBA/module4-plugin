//! Модуль для реализации логирования
//!
//! Предоставляет функциональность поддержки логирования

use std::fs::File;
use env_logger::{Builder, Target};
use log::LevelFilter;
use std::io::Write;
use std::path::Path;

/// ```rust
/// Initializes a logger with a given log level and outputs logs to a specified file.
///
/// This function sets up a logger using the `env_logger` crate's `Builder`. The logger will
/// output messages in a specific format, including timestamps, log levels, file names, line
/// numbers, and the log message itself. If the specified log file does not exist, it will
/// be created. If it exists, it will be opened for logging.
///
/// # Arguments
///
/// * `level` - A `LevelFilter` that specifies the logging level (e.g., `LevelFilter::Info`, 
///             `LevelFilter::Error`). This determines the minimum level of logs that will
///             be recorded.
/// * `file` - A string slice representing the path to the log file where logs will be written.
///
/// # Panics
///
/// This function will panic in the following scenarios:
///
/// - If the function fails to open an existing log file.
/// - If the function fails to create a new log file.
///
/// # Log Format
///
/// Logs will be formatted as:
///
/// ```txt
/// YYYY-MM-DD HH:MM:SS.mmm [LOG_LEVEL] file_name:line_number - log_message
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
/// ```rust
/// use log::LevelFilter;
///
/// setup_logger(LevelFilter::Info, "app.log");
/// log::info!("This is an informational message.");
/// ```
///
/// After calling `setup_logger`, all logs at or above the specified level will be written
/// to the provided log file with the specified format.
/// ```
pub fn setup_logger(level: LevelFilter, file: &str) {
    let log_file = {
        if Path::new(file).exists() { 
            File::open(file).expect("Error open log file")
        } else {
            File::create(file).expect("Error create log file")
        }
    };
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
}

/// ```rust
/// Converts a string representation of a log level into a `LevelFilter`.
///
/// # Arguments
///
/// * `config_str` - A string slice that specifies the desired log level. 
///   Accepted values are:
///   - `"error"`: Translated to `LevelFilter::Error`.
///   - `"warn"`: Translated to `LevelFilter::Warn`.
///   - `"info"`: Translated to `LevelFilter::Info`.
///   - `"debug"`: Translated to `LevelFilter::Debug`.
///   - `"trace"`: Translated to `LevelFilter::Trace`.
///
/// # Returns
///
/// * A `LevelFilter` corresponding to the provided string. 
///   If the input string does not match any of the predefined levels, 
///   the function defaults to returning `LevelFilter::Error`.
///
/// # Examples
///
/// ```rust
/// use log::LevelFilter;
///
/// let level = get_log_level("info");
/// assert_eq!(level, LevelFilter::Info);
///
/// let level = get_log_level("unknown");
/// assert_eq!(level, LevelFilter::Error); // Defaults to Error
/// ```
///
/// # Notes
///
/// This function performs a case-sensitive match; ensure the input string
/// matches the expected format exactly.
/// ```
pub fn get_log_level(config_str: &str) -> LevelFilter {
    match config_str.as_ref() {
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        _ => LevelFilter::Error,
    }
}
