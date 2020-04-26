use serde::{Deserialize, Serialize};

use crate::types::{Params, Version};

/// Request Id
pub type RequestId = usize;

/// Subscription Id
pub type SubscriptionId = usize;

/// Represents JSON-RPC request which is a method call.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MethodCall {
    /// A String specifying the version of the JSON-RPC protocol.
    pub jsonrpc: Option<Version>,
    /// A String containing the name of the method to be invoked.
    pub method: String,
    /// A Structured value that holds the parameter values to be used
    /// during the invocation of the method. This member MAY be omitted.
    #[serde(default = "default_params")]
    pub params: Params,
    /// An identifier established by the Client that MUST contain a String,
    /// Number, or NULL value if included. If it is not included it is assumed
    /// to be a notification.
    pub id: RequestId,
}

/// Represents JSON-RPC request which is a notification.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Notification {
    /// A String specifying the version of the JSON-RPC protocol.
    pub jsonrpc: Option<Version>,
    /// A String containing the name of the method to be invoked.
    pub method: String,
    /// A Structured value that holds the parameter values to be used
    /// during the invocation of the method. This member MAY be omitted.
    #[serde(default = "default_params")]
    pub params: Params,
}

/// Represents single JSON-RPC call.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Call {
    /// Call method
    MethodCall(MethodCall),
    /// Fire notification
    Notification(Notification),
}

impl From<MethodCall> for Call {
    fn from(call: MethodCall) -> Self {
        Call::MethodCall(call)
    }
}

impl From<Notification> for Call {
    fn from(notify: Notification) -> Self {
        Call::Notification(notify)
    }
}

/// Represents jsonrpc request.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum Request {
    /// Single request (call)
    Single(Call),
    /// Batch of requests (calls)
    Batch(Vec<Call>),
}

fn default_params() -> Params {
    Params::None
}
