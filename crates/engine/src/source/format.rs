use serde::{Deserialize, Serialize};

use crate::value::Value;

pub trait Formatter {
    type Error;
    fn format(&self, value: impl AsRef<str>) -> Result<Value, Self::Error>;
}
#[derive(Debug, Serialize, Deserialize, Default, Clone, Copy)]
pub enum Format {
    #[default]
    Raw,
    Json,
    Toml,
}
impl Formatter for Format {
    type Error = crate::Error;

    fn format(&self, value: impl AsRef<str>) -> Result<Value, Self::Error> {
        match self {
            Format::Raw => Ok(Value::String(value.as_ref().to_owned())),
            Format::Json => Ok(serde_json::from_str(value.as_ref())?),
            Format::Toml => Ok(toml::from_str(value.as_ref())?),
        }
    }
}
