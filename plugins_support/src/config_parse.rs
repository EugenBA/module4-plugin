use serde;
use crate::error::Error;

pub struct ConfigReader<T>{
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