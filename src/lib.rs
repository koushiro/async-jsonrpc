//! An asynchronous JSON-RPC client library, which supports HTTP and WebSocket.

#![deny(missing_docs)]

#[macro_use]
extern crate log;

mod errors;
mod transports;
mod types;

pub use self::errors::{Result, RpcError};
#[cfg(feature = "http-reqwest")]
pub use self::transports::HttpReqwestTransport;
#[cfg(feature = "http-surf")]
pub use self::transports::HttpSurfTransport;
pub use self::transports::{BatchTransport, PubsubTransport, Transport};
#[cfg(feature = "ws")]
pub use self::transports::{NotificationStream, WebSocketTransport};
pub use self::types::*;
