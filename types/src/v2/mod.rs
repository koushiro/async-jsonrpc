/// JSON-RPC 2.0 error objects.
pub mod error;
/// JSON-RPC 2.0 notification.
pub mod notification;
/// JSON-RPC 2.0 request/notification parameters.
pub mod params;
/// JSON-RPC 2.0 request objects.
pub mod request;
/// JSON-RPC 2.0 response objects.
pub mod response;

// Re-exports
pub use serde_json::{Map, Value};

pub use self::{
    error::{Error, ErrorCode},
    notification::{Notification, SubscriptionNotification, SubscriptionNotificationParams},
    params::{Id, Params, ParamsRef, Version},
    request::{BatchRequest, Request, RequestObj},
    response::{BatchResponse, Failure, Response, ResponseObj, Success},
};
