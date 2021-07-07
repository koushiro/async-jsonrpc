#[cfg(not(feature = "std"))]
use alloc::string::String;
use core::fmt;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Represents JSON-RPC request/response id.
///
/// An identifier established by the Client that MUST contain a String, Number,
/// or NULL value if included, If it is not included it is assumed to be a notification.
/// The value SHOULD normally not be Null and Numbers SHOULD NOT contain fractional parts.
///
/// The Server **MUST** reply with the same value in the Response object if included.
/// This member is used to correlate the context between the two objects.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum Id {
    /// Numeric id
    Num(u64),
    /// String id
    Str(String),
}

impl Id {
    /// If the `Id` is an Number, returns the associated number. Returns None
    /// otherwise.
    pub fn as_number(&self) -> Option<&u64> {
        match self {
            Self::Num(id) => Some(id),
            _ => None,
        }
    }

    /// If the `Id` is a String, returns the associated str. Returns None
    /// otherwise.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Str(id) => Some(id),
            _ => None,
        }
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Num(id) => write!(f, "{}", id),
            Self::Str(id) => f.write_str(id),
        }
    }
}

impl From<u64> for Id {
    fn from(id: u64) -> Self {
        Self::Num(id)
    }
}

impl From<String> for Id {
    fn from(id: String) -> Self {
        Self::Str(id)
    }
}

impl From<Id> for Value {
    fn from(id: Id) -> Self {
        match id {
            Id::Num(id) => Self::Number(id.into()),
            Id::Str(id) => Self::String(id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_serialization() {
        let cases = vec![
            (Id::Num(0), r#"0"#),
            (Id::Str("1".into()), r#""1""#),
            (Id::Str("test".into()), r#""test""#),
        ];

        for (id, expect) in cases {
            assert_eq!(serde_json::to_string(&id).unwrap(), expect);
            assert_eq!(id, serde_json::from_str(expect).unwrap());
        }

        assert_eq!(
            serde_json::to_string(&vec![Id::Num(0), Id::Str("1".to_owned()), Id::Str("test".to_owned()),]).unwrap(),
            r#"[0,"1","test"]"#
        );
    }
}
