//! An asynchronous JSON-RPC client library, which supports HTTP and WebSocket.

#![deny(missing_docs)]

mod error;
mod transports;

pub use self::error::RpcError;
pub use self::transports::{BatchTransport, Transport};

#[cfg(any(feature = "http-rt-async-std", feature = "http-rt-tokio"))]
pub use self::transports::{HttpTransport, HttpTransportBuilder};
// #[cfg(any(feature = "ws-rt-async-std", feature = "ws-rt-tokio"))]
// pub use self::transports::{NotificationStream, WebSocketTransport};

pub use jsonrpc_types::*;
