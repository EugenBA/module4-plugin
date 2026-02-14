//! Модуль для реализации парсинга параметров модуля
//!
//! Предоставляет функциональность по парсигу парметров модуля JSON

use serde;
use crate::error::Error;

/// ```rust
/// A generic structure for reading and storing configuration data.
///
/// The `ConfigReader` struct is designed to hold configuration data of any type.
/// It can be used to manage and encapsulate configurations for an application, minimizing the need
/// for global variables and allowing type-safe access to configuration values.
///
/// # Type Parameters
///
/// * `T` - The type of the configuration data.
///
/// # Fields
///
/// * `config` - A public field of type `T` that holds the configuration data.
///
/// # Examples
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
///
/// In this example, `ConfigReader` is instantiated with a custom `AppConfig` type to store application-specific configuration.
/// pub
///


/// Структура - возращает конфигурацию плагина
pub struct ConfigReader<T>{
    /// Конфигурация
    pub config: T,
}

impl<T> TryFrom<&str> for ConfigReader<T>
    where T: serde::de::DeserializeOwned
{
    type Error = Error;
    fn try_from(value: &str) -> Result<Self, Error>
    {
        Ok(Self
        {
            config: serde_json::from_str(value)?
        })
    }
}