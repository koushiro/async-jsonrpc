/// JSON-RPC 1.0 error objects.
pub mod error;
/// JSON-RPC 1.0 notification.
pub mod notification;
/// JSON-RPC 1.0 request/notification parameters.
pub mod params;
/// JSON-RPC 1.0 request objects.
pub mod request;
/// JSON-RPC 1.0 response objects.
pub mod response;

// Re-exports
pub use serde_json::Value;

pub use self::{
    error::{Error, ErrorCode},
    notification::{BatchNotification, BatchNotificationRef, Notification, NotificationRef},
    params::{Id, Params, ParamsRef},
    request::{BatchRequest, BatchRequestRef, Request, RequestObj, RequestRef, RequestRefObj},
    response::{BatchResponse, Response, ResponseObj},
};
