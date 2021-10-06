#[cfg(not(feature = "std"))]
use alloc::{collections::BTreeMap, string::String, vec::Vec};
use core::fmt;
#[cfg(feature = "std")]
use std::collections::BTreeMap;

use serde::{
    de::{self, DeserializeOwned},
    ser, Deserialize, Serialize,
};
use serde_json::{from_value, Value};

/// JSON-RPC 2.0 id object.
pub use crate::id::Id;
use crate::v2::Error;

/// Represents JSON-RPC protocol version.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Version {
    /// Represents JSON-RPC 2.0 version.
    V2_0,
}

impl ser::Serialize for Version {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        match self {
            Version::V2_0 => serializer.serialize_str("2.0"),
        }
    }
}

impl<'a> de::Deserialize<'a> for Version {
    fn deserialize<D>(deserializer: D) -> Result<Version, D::Error>
    where
        D: de::Deserializer<'a>,
    {
        deserializer.deserialize_identifier(VersionVisitor)
    }
}

struct VersionVisitor;
impl<'a> de::Visitor<'a> for VersionVisitor {
    type Value = Version;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a string")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match value {
            "2.0" => Ok(Version::V2_0),
            _ => Err(de::Error::custom("Invalid JSON-RPC protocol version")),
        }
    }
}

// ################################################################################################

/// Represents JSON-RPC 2.0 request parameters.
///
/// If present, parameters for the rpc call MUST be provided as a Structured value.
/// Either by-position through an Array or by-name through an Object.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub enum ParamsRef<'a> {
    /// Positional params (slice).
    ArrayRef(&'a [Value]),
    /// Params by name.
    MapRef(&'a BTreeMap<String, Value>),
}

impl<'a> Default for ParamsRef<'a> {
    fn default() -> Self {
        ParamsRef::ArrayRef(&[])
    }
}

impl<'a> fmt::Display for ParamsRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let json = serde_json::to_string(self).expect("`ParamsRef` is serializable");
        write!(f, "{}", json)
    }
}

impl<'a> PartialEq<Params> for ParamsRef<'a> {
    fn eq(&self, other: &Params) -> bool {
        match (other, self) {
            (Params::Array(arr1), ParamsRef::ArrayRef(arr2)) => arr1.eq(arr2),
            (Params::Map(map1), ParamsRef::MapRef(map2)) => map1.eq(map2),
            _ => false,
        }
    }
}

impl<'a> From<&'a [Value]> for ParamsRef<'a> {
    fn from(params: &'a [Value]) -> Self {
        Self::ArrayRef(params)
    }
}

impl<'a> From<&'a BTreeMap<String, Value>> for ParamsRef<'a> {
    fn from(params: &'a BTreeMap<String, Value>) -> Self {
        Self::MapRef(params)
    }
}

impl<'a> ParamsRef<'a> {
    /// Checks if the parameters is an empty array of objects.
    pub fn is_empty_array(&self) -> bool {
        matches!(self, ParamsRef::ArrayRef(array) if array.is_empty())
    }

    /// Checks if the parameters is an array of objects.
    pub fn is_array(&self) -> bool {
        matches!(self, ParamsRef::ArrayRef(_))
    }

    /// Checks if the parameters is a map of objects.
    pub fn is_map(&self) -> bool {
        matches!(self, ParamsRef::MapRef(_))
    }

    /// Converts the reference into the owned type.
    pub fn to_owned(&self) -> Params {
        match *self {
            Self::ArrayRef(params) => Params::Array(params.to_vec()),
            Self::MapRef(params) => Params::Map(params.clone()),
        }
    }
}

// ################################################################################################

/// Represents JSON-RPC 2.0 request parameters.
///
/// If present, parameters for the rpc call MUST be provided as a Structured value.
/// Either by-position through an Array or by-name through an Object.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Params {
    /// Positional params (heap allocated).
    Array(Vec<Value>),
    /// Params by name.
    Map(BTreeMap<String, Value>),
}

impl Default for Params {
    fn default() -> Self {
        Params::Array(Vec::new())
    }
}

impl fmt::Display for Params {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let json = serde_json::to_string(self).expect("`Params` is serializable");
        write!(f, "{}", json)
    }
}

impl<'a> PartialEq<ParamsRef<'a>> for Params {
    fn eq(&self, other: &ParamsRef<'a>) -> bool {
        match (self, other) {
            (Params::Array(arr1), ParamsRef::ArrayRef(arr2)) => arr1.eq(arr2),
            (Params::Map(map1), ParamsRef::MapRef(map2)) => map1.eq(map2),
            _ => false,
        }
    }
}

impl From<Vec<Value>> for Params {
    fn from(params: Vec<Value>) -> Self {
        Self::Array(params)
    }
}

impl From<BTreeMap<String, Value>> for Params {
    fn from(params: BTreeMap<String, Value>) -> Self {
        Self::Map(params)
    }
}

impl From<Params> for Value {
    fn from(params: Params) -> Value {
        match params {
            Params::Array(array) => Value::Array(array),
            Params::Map(object) => Value::Object(object.into_iter().collect()),
        }
    }
}

impl Params {
    /// Parses incoming `Params` into expected types.
    pub fn parse<D>(self) -> Result<D, Error>
    where
        D: DeserializeOwned,
    {
        let value = self.into();
        from_value(value).map_err(Error::invalid_params)
    }

    /// Checks if the parameters is an empty array of objects.
    pub fn is_empty_array(&self) -> bool {
        matches!(self, Params::Array(array) if array.is_empty())
    }

    /// Checks if the parameters is an array of objects.
    pub fn is_array(&self) -> bool {
        matches!(self, Params::Array(_))
    }

    /// Checks if the parameters is a map of objects.
    pub fn is_map(&self) -> bool {
        matches!(self, Params::Map(_))
    }

    /// Borrows from an owned value.
    pub fn as_ref(&self) -> ParamsRef<'_> {
        match self {
            Self::Array(params) => ParamsRef::ArrayRef(params),
            Self::Map(params) => ParamsRef::MapRef(params),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn params_serialization() {
        let array = vec![Value::from(1), Value::Bool(true)];
        let params = Params::Array(array.clone());
        assert_eq!(serde_json::to_string(&params).unwrap(), r#"[1,true]"#);
        assert_eq!(serde_json::to_string(&params.as_ref()).unwrap(), r#"[1,true]"#);
        assert_eq!(serde_json::from_str::<Params>(r#"[1,true]"#).unwrap(), params);

        let object = {
            let mut map = BTreeMap::new();
            map.insert("key".into(), Value::String("value".into()));
            map
        };
        let params = Params::Map(object.clone());
        assert_eq!(serde_json::to_string(&params).unwrap(), r#"{"key":"value"}"#);
        assert_eq!(serde_json::to_string(&params.as_ref()).unwrap(), r#"{"key":"value"}"#);
        assert_eq!(serde_json::from_str::<Params>(r#"{"key":"value"}"#).unwrap(), params);

        let params = Params::Array(vec![
            Value::Null,
            Value::Bool(true),
            Value::from(-1),
            Value::from(1),
            Value::from(1.2),
            Value::String("hello".into()),
            Value::Array(vec![]),
            Value::Array(array),
            Value::Object(object.into_iter().collect()),
        ]);
        assert_eq!(
            serde_json::to_string(&params).unwrap(),
            r#"[null,true,-1,1,1.2,"hello",[],[1,true],{"key":"value"}]"#
        );
        assert_eq!(
            serde_json::to_string(&params.as_ref()).unwrap(),
            r#"[null,true,-1,1,1.2,"hello",[],[1,true],{"key":"value"}]"#
        );
        assert_eq!(
            serde_json::from_str::<Params>(r#"[null,true,-1,1,1.2,"hello",[],[1,true],{"key":"value"}]"#).unwrap(),
            params
        );
    }

    #[test]
    fn single_param_parsed_as_tuple() {
        let params: (u64,) = Params::Array(vec![Value::from(1)]).parse().unwrap();
        assert_eq!(params, (1,));
    }

    #[test]
    fn invalid_params() {
        let params = serde_json::from_str::<Params>("[1,true]").unwrap();
        assert_eq!(
            params.parse::<(u8, bool, String)>().unwrap_err(),
            Error::invalid_params("invalid length 2, expected a tuple of size 3")
        );
    }
}
