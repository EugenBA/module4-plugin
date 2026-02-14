//! Модуль для реализации логирования
//!
//! Предоставляет функциональность по оработе с логами

use std::fs::File;
use env_logger::{Builder, Target};
use log::LevelFilter;
use std::io::Write;
use std::path::Path;

pub(crate) fn setup_logger(level: LevelFilter, file: &str) {
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

pub(crate) fn get_logger_filter(log_level: &str) -> LevelFilter{
    match log_level.as_ref() {
        "debug" => LevelFilter::Debug,
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "trace" => LevelFilter::Trace,
        _ => LevelFilter::Info,
    }
}