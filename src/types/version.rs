use serde::{self, Serialize, Deserialize};

/// Protocol Version
#[derive(Debug, PartialEq, Clone, Copy, Hash, Eq, Serialize, Deserialize)]
pub enum Version {
    /// JSON-RPC 2.0
    #[serde(rename = "2.0")]
    V2,
}
