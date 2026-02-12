use clap::{Parser};


#[derive(Parser, Debug)]
#[command(name = "image-processor")]
#[command(about = "Image processor application", version = "1.0")]
pub(crate) struct Cli{
    /// Path image file
    #[arg(long)]
    pub(crate) input: String,
    /// Path store converted image
    #[arg(long)]
    pub(crate) output: String,
    /// Name plugin
    #[arg(long)]
    pub(crate) plugin: String,
    /// Path config file for plugin
    #[arg(long)]
    pub(crate) params:String,
    /// Plugin directory path
    #[arg(long)]
    pub(crate) plugin_path: String,
    /// Log level (info, warn, error, debug, trace), default info
    #[arg(long, default_value = "error", required=false)]
    pub(crate) log_level: String
}