use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum ImageProcessorError
{
    #[error("Path not exists: {0}")]
    PathNotExist(#[from] io::Error),
    #[error("Convertation error: {0}")]
    ImageReadError(#[from] image::ImageError),
    #[error("Library loading error: {0}")]
    PluginError(#[from] libloading::Error),
    #[error("Error convert image from raw")]
    ConvertFromRawError
}