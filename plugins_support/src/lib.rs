//! Модуль для реализации общего функционала плагинов
//!
//! Предоставляет общую функциональность: логирование, парсинг конфигурации, ошибки

#![warn(missing_docs)]
pub mod config_parse;
pub mod error;
pub mod logger;


#[cfg(test)]
mod tests {
    use serde::Deserialize;
    use crate::config_parse::ConfigReader;
    use crate::error::Error;
    #[test]
    fn test_parse_json()  {
        #[derive(Deserialize, Debug)]
        struct ConfigTransform {
            step: usize,
        }
        let json_str = r#"{"step": 1}"#;
        let config: Result<ConfigReader<ConfigTransform>, Error> = ConfigReader::try_from(json_str);
        assert_eq!(config.unwrap().config.step, 1);
    }
}
