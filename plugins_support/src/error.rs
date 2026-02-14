//! Модуль для реализации обработки ошибок
//!
//! Предоставляет функциональность по обработке ошибок
use thiserror::Error;


/// ```
/// An enumeration representing possible errors that can occur during processing.
///
/// This enum leverages the [`thiserror`] crate to provide custom error messages
/// for each variant. It also derives the [`Debug`] trait for enhanced debugging support.
///
/// # Variants
///
/// * `NullPointerRGBABuffer`
///     - Represents an error condition where a NULL pointer in the RGBA buffer is encountered.
///     - Error message: `"Null pointer RGBA buffer"`.
///
/// * `NullPointerParams`
///     - Represents an error condition where a NULL pointer in the parameters is encountered.
///     - Error message: `"Null pointer params"`.
///
/// * `ParamsAreNotValidJSON`
///     - Represents an error when the provided parameters are not valid JSON.
///     - This variant contains an inner [`serde_json::Error`] to provide details about JSON parsing errors.
///     - Error message: `"Params are not valid JSON {0}"`.
///
/// # Usage
///
/// This error enum is typically used in scenarios involving image processing or
/// parameter validation where pointers and JSON parsing are involved.
///
/// # Example
/// ```
/// use crate::Error;
///
/// // Example of creating an error for invalid JSON parameters.
/// let invalid_json_error = Error::ParamsAreNotValidJSON(
///     serde_json::from_str::<serde_json::Value>("invalid_json").unwrap_err()
/// );
///
/// // Example of a null RGBA buffer error.
/// let rgba_error = Error::NullPointerRGBABuffer;
///
/// // Example of a null params error.
/// let params_error = Error::NullPointerParams;
/// ```
///
/// # Dependencies
///
/// This code assumes the following crates are included:
/// - `thiserror`
/// - `serde_json` for handling JSON parsing errors.
/// ```
#[derive(Error, Debug)]
pub enum Error{
    /// Ошибка обработки нулевого указателя RGBA буффера
    #[error("Null pointer RGBA buffer")]
    NullPointerRGBABuffer,
    /// Ошибка нулевого указателя на конфигурации модуля
    #[error("Null pointer params")]
    NullPointerParams,
    /// Ошибка парсинга JSON строки параметров
    #[error("Params are not valid JSON {0}")]
    ParamsAreNotValidJSON(#[from] serde_json::Error)
}
