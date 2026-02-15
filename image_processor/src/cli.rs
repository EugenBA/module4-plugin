//! Модуль для обработки командной строки
//!
//! Предоставляет функциональность парметров командной строки

use std::path::PathBuf;
use clap::Parser;
//output: PathBuf
#[derive(Parser, Debug)]
#[command(name = "image-processor")]
#[command(about = "Image processor application", version = "1.0")]
pub(crate) struct Cli {
    /// Path image file
    #[arg(long)]
    pub(crate) input: PathBuf,
    /// Path store converted image
    #[arg(long)]
    pub(crate) output: PathBuf,
    /// Name plugin
    #[arg(long)]
    pub(crate) plugin: PathBuf,
    /// Path config file for plugin
    #[arg(long)]
    pub(crate) params: PathBuf,
    /// Plugin directory path
    #[arg(long)]
    pub(crate) plugin_path: PathBuf,
    /// Log level (info, warn, error, debug, trace), default info
    #[arg(long, default_value = "error", required = false)]
    pub(crate) log_level: String,
}
