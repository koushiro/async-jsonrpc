use std::fmt;

use serde::{de, ser};

/// JSON-RPC Protocol Version.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Version {
    /// JSON-RPC 2.0
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
