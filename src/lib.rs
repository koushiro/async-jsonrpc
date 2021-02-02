//! An asynchronous JSON-RPC 2.0 client library, which supports HTTP and WebSocket.

#![deny(missing_docs)]

mod error;
mod transport;

mod http_client;
mod ws_client;

pub use self::{
    error::RpcClientError,
    transport::{BatchTransport, PubsubTransport, Transport},
};

pub use self::http_client::{HttpTransport, HttpTransportBuilder};
pub use self::ws_client::{WsSubscription, WsTransport, WsTransportBuilder};

pub use http::header::{self, HeaderName, HeaderValue};
pub use jsonrpc_types::*;

/*
//#[cfg(any(feature = "http-async-std", feature = "http-tokio"))]
#[cfg(feature = "http-tokio")]
pub use self::transports::{HttpTransport, HttpTransportBuilder};
// #[cfg(any(feature = "ws-async-std", feature = "ws-tokio"))]
#[cfg(feature = "ws-tokio")]
pub use self::transports::{NotificationStream, WsTransport, WsTransportBuilder};
*/
