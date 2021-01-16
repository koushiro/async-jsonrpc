//! An asynchronous JSON-RPC client library, which supports HTTP and WebSocket.

#![deny(missing_docs)]

//#[macro_use]
//extern crate log;

mod error;
// mod transports;

pub use self::error::RpcError;
// pub use self::transports::{BatchTransport, PubsubTransport, Transport};

//#[cfg(any(feature = "http-rt-tokio"), feature = "http-rt-async-std")]
//pub use self::transports::HttpTransport;
//#[cfg(any(feature = "ws-rt-tokio"), feature = "ws-rt-async-std")]
//pub use self::transports::{NotificationStream, WebSocketTransport};

pub use jsonrpc_types::*;
