//! Модуль для реализации парсинга параметров модуля
//!
//! Предоставляет функциональность по парсигу парметров модуля JSON

use crate::error::Error;
use serde;

/// ```rust
/// Дженерик структура для конфигурационных данных JSON
///
///
/// # Тип
///
/// * `T` - тип для конфигурационных данных
///
/// # Поля
///
/// * `config` - публичное поле тип
///
/// # Пример
///
/// ```rust
/// struct AppConfig {
///     host: String,
///     port: u16,
/// }
///
/// let app_config = AppConfig {
///     host: "localhost".to_string(),
///     port: 8080,
/// };
///
/// let config_reader = ConfigReader { config: app_config };
///
/// println!("Host: {}, Port: {}", config_reader.config.host, config_reader.config.port);
/// ```

/// Структура - возращает конфигурацию плагина
pub struct ConfigReader<T> {
    /// Конфигурация
    pub config: T,
}

impl<T> TryFrom<&str> for ConfigReader<T>
where
    T: serde::de::DeserializeOwned,
{
    type Error = Error;
    fn try_from(value: &str) -> Result<Self, Error> {
        Ok(Self {
            config: serde_json::from_str(value)?,
        })
    }
}
