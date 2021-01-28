//! An asynchronous JSON-RPC client library, which supports HTTP and WebSocket.

#![deny(missing_docs)]

mod error;
mod transports;

pub use self::error::RpcClientError;
pub use self::transports::{BatchTransport, NotificationStream, PubsubTransport, Transport};

//#[cfg(any(feature = "http-async-std", feature = "http-tokio"))]
#[cfg(feature = "http-tokio")]
pub use self::transports::{HttpTransport, HttpTransportBuilder};
// #[cfg(any(feature = "ws-async-std", feature = "ws-tokio"))]
#[cfg(feature = "ws-tokio")]
pub use self::transports::{WsTransport, WsTransportBuilder};

pub use jsonrpc_types::*;
