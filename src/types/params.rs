use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Map as JsonMap;

use crate::types::{Error, Value};

/// Request parameters
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Params {
    /// No parameters
    None,
    /// Array of values
    Array(Vec<Value>),
    /// Map of values
    Map(JsonMap<String, Value>),
}

impl Params {
    /// Parse incoming `Params` into expected types.
    pub fn parse<D>(self) -> Result<D, Error>
    where
        D: DeserializeOwned,
    {
        let value = match self {
            Params::Array(vec) => Value::Array(vec),
            Params::Map(map) => Value::Object(map),
            Params::None => Value::Null,
        };

        serde_json::from_value(value)
            .map_err(|err| Error::invalid_params(format!("Invalid params: {}.", err)))
    }

    /// Check for no params, returns Err if any params
    pub fn expect_no_params(self) -> Result<(), Error> {
        match self {
            Params::None => Ok(()),
            Params::Array(ref v) if v.is_empty() => Ok(()),
            p => Err(Error::invalid_params_with_details(
                "No parameters were expected",
                p,
            )),
        }
    }
}

impl From<Params> for Value {
    fn from(params: Params) -> Value {
        match params {
            Params::Array(vec) => Value::Array(vec),
            Params::Map(map) => Value::Object(map),
            Params::None => Value::Null,
        }
    }
}
