use serde::Deserialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum Error{
    #[error("Null pointer RGBA buffer")]
    NullPointerRGBABuffer,
    #[error("Null pointer params")]
    NullPointerParams,
    #[error("Params are not valid JSON {0}")]
    ParamsAreNotValidJSON(#[from] serde_json::Error)

}