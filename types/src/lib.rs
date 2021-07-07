//! A set of types for representing JSON-RPC requests and responses as defined in
//! the [JSON-RPC 1.0 spec](https://www.jsonrpc.org/specification_v1) and
//! [JSON-RPC 2.0 spec](https://www.jsonrpc.org/specification).
//!
//! # Usage
//! ## Creates JSON-RPC 1.0 request
//!
#![cfg_attr(feature = "v1", doc = r##"
```rust
use jsonrpc_types::v1::{Call, MethodCall, Notification, Request};

// Creates a JSON-RPC 1.0 method call request
let method_call = MethodCall::new("foo", vec![], 1.into());
let method_call_req = Request::Single(Call::MethodCall(method_call));
assert_eq!(
    serde_json::to_string(&method_call_req).unwrap(),
    r#"{"method":"foo","params":[],"id":1}"#
);

// Creates a JSON-RPC 1.0 notification request
let notification = Notification::new("foo", vec![]);
let notification_req = Request::Single(Call::Notification(notification.clone()));
assert_eq!(
    serde_json::to_string(&notification_req).unwrap(),
    r#"{"method":"foo","params":[],"id":null}"#
);

// Creates a JSON-RPC 1.0 batch request
let batch_request = Request::Batch(vec![
    Call::MethodCall(MethodCall::new("foo", vec![], 1.into())),
    Call::MethodCall(MethodCall::new("bar", vec![], 2.into())),
]);
assert_eq!(
    serde_json::to_string(&batch_request).unwrap(),
    r#"[{"method":"foo","params":[],"id":1},{"method":"bar","params":[],"id":2}]"#
);
```
"##)]
//!
//! ## Creates JSON-RPC 1.0 response
//!
#![cfg_attr(feature = "v1", doc = r##"
```rust
use jsonrpc_types::v1::{Value, Error, Output, Response};
// Creates a JSON-RPC 1.0 success response
let success_response = Output::success(Value::Bool(true), 1.into());
let response1 = Response::Single(success_response.clone());
assert_eq!(
    serde_json::to_string(&response1).unwrap(),
    r#"{"result":true,"error":null,"id":1}"#
);
// Creates a JSON-RPC 1.0 failure response
let failure_response = Output::<Value>::failure(Error::invalid_request(), None);
let response2 = Response::Single(failure_response.clone());
assert_eq!(
    serde_json::to_string(&response2).unwrap(),
    r#"{"result":null,"error":{"code":-32600,"message":"Invalid request"},"id":null}"#
);
// Creates a JSON-RPC 1.0 batch response
let success1 = Output::success(Value::Bool(true), 1.into());
let success2 = Output::success(Value::Bool(false), 2.into());
let batch_response = Response::Batch(vec![success1, success2]);
assert_eq!(
    serde_json::to_string(&batch_response).unwrap(),
    r#"[{"result":true,"error":null,"id":1},{"result":false,"error":null,"id":2}]"#
);
```
"##)]
//!
//! ## Creates JSON-RPC 2.0 request
//!
#![cfg_attr(feature = "v2", doc = r##"
```rust
use jsonrpc_types::v2::{Params, MethodCall, Notification, Call, Request};

// Creates a JSON-RPC 2.0 method call request
let method_call = MethodCall::new("foo", Some(Params::Array(vec![])), 1.into());
let method_call_req = Request::Single(Call::MethodCall(method_call));
assert_eq!(
    serde_json::to_string(&method_call_req).unwrap(),
    r#"{"jsonrpc":"2.0","method":"foo","params":[],"id":1}"#
);

// Creates a JSON-RPC 2.0 notification request
let notification = Notification::new("foo", Some(Params::Array(vec![])));
let notification_req = Request::Single(Call::Notification(notification.clone()));
assert_eq!(
    serde_json::to_string(&notification_req).unwrap(),
    r#"{"jsonrpc":"2.0","method":"foo","params":[]}"#
);

// Creates a JSON-RPC 2.0 batch request
let batch_request = Request::Batch(vec![
    Call::MethodCall(MethodCall::new("foo", Some(Params::Array(vec![])), 1.into())),
    Call::MethodCall(MethodCall::new("bar", Some(Params::Array(vec![])), 2.into())),
]);
assert_eq!(
    serde_json::to_string(&batch_request).unwrap(),
    r#"[{"jsonrpc":"2.0","method":"foo","params":[],"id":1},{"jsonrpc":"2.0","method":"bar","params":[],"id":2}]"#
);
```
"##)]
//!
//! ## Creates JSON-RPC 2.0 response
//!
#![cfg_attr(feature = "v2", doc = r##"
```rust
use jsonrpc_types::v2::{Value, Error, Success, Failure, Output, Response};

// Creates a JSON-RPC 2.0 success response
let success = Success::new(Value::Bool(true), 1.into());
let response1 = Response::Single(Output::Success(success.clone()));
assert_eq!(
    serde_json::to_string(&response1).unwrap(),
    r#"{"jsonrpc":"2.0","result":true,"id":1}"#
);

// Creates a JSON-RPC 2.0 failure response
let failure = Failure::new(Error::invalid_request(), None);
let response2 = Response::<Value>::Single(Output::Failure(failure.clone()));
assert_eq!(
    serde_json::to_string(&response2).unwrap(),
    r#"{"jsonrpc":"2.0","error":{"code":-32600,"message":"Invalid request"},"id":null}"#
);

// Creates a JSON-RPC 2.0 batch response
let success1 = Output::success(Value::Bool(true), 1.into());
let success2 = Output::success(Value::Bool(false), 2.into());
let batch_response = Response::Batch(vec![success1, success2]);
assert_eq!(
    serde_json::to_string(&batch_response).unwrap(),
    r#"[{"jsonrpc":"2.0","result":true,"id":1},{"jsonrpc":"2.0","result":false,"id":2}]"#
);
```
"##)]
//!
//! # Crate features
//!
//! **v2** features are enabled by default.
//!
//! ## JSON-RPC version features
//!
//! * **v1** -
//!   Provide the JSON-RPC 1.0 types.
//! * **v2** -
//!   Provide the JSON-RPC 2.0 types.
//!

#![deny(unused_imports)]
#![deny(missing_docs)]
#![cfg_attr(doc_cfg, feature(doc_cfg))]

/// JSON-RPC 1.0 types.
#[cfg(feature = "v1")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "v1")))]
pub mod v1;
/// JSON-RPC 2.0 types.
#[cfg(feature = "v2")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "v2")))]
pub mod v2;

#[cfg(any(feature = "v1", feature = "v2"))]
mod error;
#[cfg(any(feature = "v1", feature = "v2"))]
mod id;
