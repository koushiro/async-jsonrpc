#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use serde_json::Value;

/// JSON-RPC 1.0 id object.
pub use crate::id::Id;

/// Represents JSON-RPC 1.0 request parameters.
pub type Params = Vec<Value>;

/// Represents JSON-RPC 1.0 request parameters.
pub type ParamsRef<'a> = &'a [Value];
