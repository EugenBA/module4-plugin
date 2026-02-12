use clap::{Parser};


#[derive(Parser, Debug)]
#[command(name = "image-processor")]
#[command(about = "Image processor application", version = "1.0")]
pub(crate) struct Cli{
    ///
    #[arg(long)]
    pub(crate) input: String,
    #[arg(long)]
    pub(crate) output: String,
    #[arg(long)]
    pub(crate) plugin: String,
    #[arg(long)]
    pub(crate) params:String,
    #[arg(long)]
    pub(crate) plugin_path: String,
    #[arg(long)]
    pub(crate) log_level: String
}