//! A set of types for representing JSON-RPC requests and responses as defined in
//! the [JSON-RPC 1.0 spec](https://www.jsonrpc.org/specification_v1) and
//! [JSON-RPC 2.0 spec](https://www.jsonrpc.org/specification).
//!
//! # Usage
#![cfg_attr(
    feature = "v1",
    doc = r##"
## Creates JSON-RPC 1.0 request

```rust
use jsonrpc_types::v1::{Notification, Request, RequestObj};

// Creates a JSON-RPC 1.0 request call
let request = Request::new("foo", vec![], 1.into());
let request = RequestObj::Single(request);
assert_eq!(
    serde_json::to_string(&request).unwrap(),
    r#"{"method":"foo","params":[],"id":1}"#
);

// Creates a JSON-RPC 1.0 notification
let notification = Notification::new("foo", vec![]);
assert_eq!(
    serde_json::to_string(&notification).unwrap(),
    r#"{"method":"foo","params":[],"id":null}"#
);

// Creates a JSON-RPC 1.0 batch request
let request1 = Request::new("foo", vec![], 1.into());
let request2 = Request::new("bar", vec![], 2.into());
let batch_request = RequestObj::Batch(vec![request1, request2]);
assert_eq!(
    serde_json::to_string(&batch_request).unwrap(),
    r#"[{"method":"foo","params":[],"id":1},{"method":"bar","params":[],"id":2}]"#
);
```
"##
)]
//!
#![cfg_attr(
    feature = "v1",
    doc = r##"
## Creates JSON-RPC 1.0 response

```rust
use jsonrpc_types::v1::{Value, Error, Response, ResponseObj};

// Creates a JSON-RPC 1.0 success response
let success = Response::success(Value::Bool(true), 1.into());
let response = ResponseObj::Single(success);
assert_eq!(
    serde_json::to_string(&response).unwrap(),
    r#"{"result":true,"error":null,"id":1}"#
);

// Creates a JSON-RPC 1.0 failure response
let failure = Response::<Value>::failure(Error::invalid_request(), None);
let response = ResponseObj::Single(failure);
assert_eq!(
    serde_json::to_string(&response).unwrap(),
    r#"{"result":null,"error":{"code":-32600,"message":"Invalid request"},"id":null}"#
);

// Creates a JSON-RPC 1.0 batch response
let success1 = Response::success(Value::Bool(true), 1.into());
let success2 = Response::success(Value::Bool(false), 2.into());
let batch_response = ResponseObj::Batch(vec![success1, success2]);
assert_eq!(
    serde_json::to_string(&batch_response).unwrap(),
    r#"[{"result":true,"error":null,"id":1},{"result":false,"error":null,"id":2}]"#
);
```
"##
)]
//!
#![cfg_attr(
    feature = "v2",
    doc = r##"
## Creates JSON-RPC 2.0 request

```rust
use jsonrpc_types::v2::{Params, Request, RequestObj, Notification};

// Creates a JSON-RPC 2.0 request call
let request = Request::new("foo", Some(Params::Array(vec![])), 1.into());
let request = RequestObj::Single(request);
assert_eq!(
    serde_json::to_string(&request).unwrap(),
    r#"{"jsonrpc":"2.0","method":"foo","params":[],"id":1}"#
);

// Creates a JSON-RPC 2.0 notification
let notification = Notification::new("foo", Some(Params::Array(vec![])));
assert_eq!(
    serde_json::to_string(&notification).unwrap(),
    r#"{"jsonrpc":"2.0","method":"foo","params":[]}"#
);

// Creates a JSON-RPC 2.0 batch request
let request1 = Request::new("foo", Some(Params::Array(vec![])), 1.into());
let request2 = Request::new("bar", Some(Params::Array(vec![])), 2.into());
let batch_request = RequestObj::Batch(vec![request1, request2]);
assert_eq!(
    serde_json::to_string(&batch_request).unwrap(),
    r#"[{"jsonrpc":"2.0","method":"foo","params":[],"id":1},{"jsonrpc":"2.0","method":"bar","params":[],"id":2}]"#
);
```
"##
)]
//!
#![cfg_attr(
    feature = "v2",
    doc = r##"
## Creates JSON-RPC 2.0 response

```rust
use jsonrpc_types::v2::{Value, Error, Success, Failure, Response, ResponseObj};

// Creates a JSON-RPC 2.0 success response
let success = Response::success(Value::Bool(true), 1.into());
let response = ResponseObj::Single(success);
assert_eq!(
    serde_json::to_string(&response).unwrap(),
    r#"{"jsonrpc":"2.0","result":true,"id":1}"#
);

// Creates a JSON-RPC 2.0 failure response
let failure = Response::<Value>::failure(Error::invalid_request(), None);
let response = ResponseObj::Single(failure);
assert_eq!(
    serde_json::to_string(&response).unwrap(),
    r#"{"jsonrpc":"2.0","error":{"code":-32600,"message":"Invalid request"},"id":null}"#
);

// Creates a JSON-RPC 2.0 batch response
let success1 = Response::success(Value::Bool(true), 1.into());
let success2 = Response::success(Value::Bool(false), 2.into());
let batch_response = ResponseObj::Batch(vec![success1, success2]);
assert_eq!(
    serde_json::to_string(&batch_response).unwrap(),
    r#"[{"jsonrpc":"2.0","result":true,"id":1},{"jsonrpc":"2.0","result":false,"id":2}]"#
);
```
"##
)]
//!
//! # Crate features
//!
//! **std** and **v2** features are enabled by default.
//!
//! ## Ecosystem features
//!
//! * **std** -
//!   When enabled, this crate will use the standard library.
//!   Currently, disabling this feature will always use `alloc` library.
//!
//! ## JSON-RPC version features
//!
//! * **v1** -
//!   Provide the JSON-RPC 1.0 types.
//! * **v2** -
//!   Provide the JSON-RPC 2.0 types.

#![deny(unused_imports)]
#![deny(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(doc_cfg, feature(doc_cfg))]

#[cfg(not(feature = "std"))]
extern crate alloc;

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
