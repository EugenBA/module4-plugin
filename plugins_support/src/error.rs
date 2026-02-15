//! Модуль для реализации обработки ошибок
//!
//! Предоставляет функциональность по обработке ошибок
use thiserror::Error;

///
/// Перечисление для прдесталения ошибок
///
///
/// # Варианты
///
/// * `NullPointerRGBABuffer`
///     - Ошибка нулевого указателя RGBA буфера
///     - Error message: `"Null pointer RGBA buffer"`.
///
/// * `NullPointerParams`
///     - Ошибка нулевого указателя параметров
///     - Error message: `"Null pointer params"`.
///
/// * `ParamsAreNotValidJSON`
///     - Ошибка десерилизации JSON параметров
///     - Error message: `"Params are not valid JSON {0}"`.
///
///
///
#[derive(Error, Debug)]
pub enum Error {
    /// Ошибка обработки нулевого указателя RGBA буффера
    #[error("Null pointer RGBA buffer")]
    NullPointerRGBABuffer,
    /// Ошибка нулевого указателя на конфигурации модуля
    #[error("Null pointer params")]
    NullPointerParams,
    /// Ошибка парсинга JSON строки параметров
    #[error("Params are not valid JSON {0}")]
    ParamsAreNotValidJSON(#[from] serde_json::Error),
    /// Ошибка создания файла лога
    #[error("File create error: {0}")]
    FileCreateError(#[from] std::io::Error),
    #[error("Error convert type {0}")]
    /// Ошибка преобразования типа
    ConvertTypeError(#[from] std::num::TryFromIntError),
    #[error("Error value: {0}")]
    /// Ошибка занчения
    ErrorValue(String),
    /// Переполнение типа
    #[error("Overflow type")]
    OverflowError
}
