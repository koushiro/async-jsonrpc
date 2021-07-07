/// JSON-RPC 2.0 error objects.
pub mod error;
/// JSON-RPC 2.0 id objects.
pub mod id;
/// JSON-RPC 2.0 request objects.
pub mod request;
/// JSON-RPC 2.0 response objects.
pub mod response;
/// JSON-RPC protocol version.
pub mod version;

pub use self::{
    error::{Error, ErrorCode},
    id::Id,
    request::{
        Call, MethodCall, MethodCallRequest, Notification, Params, Request, SubscriptionNotification,
        SubscriptionNotificationParams,
    },
    response::{Failure, Output, Response, Success},
    version::Version,
};

// Re-exports
pub use serde_json::{Map, Value};
