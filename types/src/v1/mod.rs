/// JSON-RPC 1.0 error objects.
pub mod error;
/// JSON-RPC 1.0 id objects.
pub mod id;
/// JSON-RPC 1.0 request objects.
pub mod request;
/// JSON-RPC 1.0 response objects.
pub mod response;

pub use self::{
    error::{Error, ErrorCode},
    id::Id,
    request::{Call, MethodCall, MethodCallRequest, Notification, Params, Request},
    response::{Output, Response},
};

// Re-exports
pub use serde_json::Value;
