//! An asynchronous JSON-RPC client library, which supports HTTP and WebSocket.

#![deny(missing_docs)]

#[macro_use]
extern crate log;

mod errors;
mod transports;
mod types;

pub use self::errors::{Result, RpcError};
pub use self::transports::{BatchTransport, PubsubTransport, Transport};
pub use self::transports::{HttpTransport, NotificationStream, WebSocketTransport};
pub use self::types::*;
