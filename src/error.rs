use thiserror::Error;

/// The error type for rpc transport.
#[cfg(any(feature = "http-async-std", feature = "http-tokio"))]
#[derive(Debug, Error)]
pub enum HttpClientError {
    /// Json serialization/deserialization error.
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    /// HTTP error.
    #[cfg(feature = "http-async-std")]
    #[error(transparent)]
    Http(#[from] anyhow::Error),

    /// HTTP error.
    #[cfg(feature = "http-tokio")]
    #[error(transparent)]
    Http(#[from] reqwest::Error),
}

/// WebSocket error type.
#[cfg(any(feature = "ws-async-std", feature = "ws-tokio"))]
pub use async_tungstenite::tungstenite::Error as WsError;

/// The error type for websocket rpc transport.
#[cfg(any(feature = "ws-async-std", feature = "ws-tokio"))]
#[derive(Debug, Error)]
pub enum WsClientError {
    /// Json serialization/deserialization error.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// WebSocket protocol error.
    #[error(transparent)]
    WebSocket(#[from] WsError),
    /// WebSocket request timeout.
    #[error("WebSocket request timeout")]
    RequestTimeout,
    /// Duplicate request ID.
    #[error("Duplicate request ID")]
    DuplicateRequestId,
    /// Invalid Request ID.
    #[error("Invalid request ID")]
    InvalidRequestId,
    /// Invalid Subscription ID.
    #[error("Invalid subscription ID")]
    InvalidSubscriptionId,
    /// Invalid Unsubscribe request result.
    #[error("Invalid Unsubscribe result")]
    InvalidUnsubscribeResult,
    /// Internal channel error
    #[error("Internal channel error")]
    InternalChannel,
}
