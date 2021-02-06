//! An asynchronous JSON-RPC 2.0 client library, which supports HTTP and WebSocket.

#![deny(missing_docs)]

mod error;
mod transport;

#[cfg(any(feature = "http-async-std", feature = "http-tokio"))]
mod http_client;
#[cfg(any(feature = "ws-async-std", feature = "ws-tokio"))]
mod ws_client;

pub use self::transport::{BatchTransport, PubsubTransport, Transport};

#[cfg(any(feature = "http-async-std", feature = "http-tokio"))]
pub use self::{
    error::HttpClientError,
    http_client::{HttpClient, HttpClientBuilder},
};
#[cfg(any(feature = "ws-async-std", feature = "ws-tokio"))]
pub use self::{
    error::{WsClientError, WsError},
    ws_client::{WsClient, WsClientBuilder, WsSubscription},
};

pub use http::header::{self, HeaderName, HeaderValue};
pub use jsonrpc_types::*;
