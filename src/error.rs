use thiserror::Error;

pub(crate) type Result<T, E = ClientError> = std::result::Result<T, E>;

/// WebSocket error type.
#[cfg(any(feature = "ws-async-std", feature = "ws-tokio"))]
pub use async_tungstenite::tungstenite::Error as WsError;

/// The error type for rpc transport.
#[derive(Debug, Error)]
pub enum ClientError {
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

    /// WebSocket protocol error.
    #[cfg(any(feature = "ws-async-std", feature = "ws-tokio"))]
    #[error(transparent)]
    WebSocket(#[from] WsError),
    /// WebSocket request timeout.
    #[cfg(any(feature = "ws-async-std", feature = "ws-tokio"))]
    #[error("WebSocket request timeout")]
    WsRequestTimeout,
    /// Duplicate request ID.
    #[cfg(any(feature = "ws-async-std", feature = "ws-tokio"))]
    #[error("Duplicate request ID")]
    DuplicateRequestId,
    /// Invalid Request ID.
    #[cfg(any(feature = "ws-async-std", feature = "ws-tokio"))]
    #[error("Invalid request ID")]
    InvalidRequestId,
    /// Invalid Subscription ID.
    #[cfg(any(feature = "ws-async-std", feature = "ws-tokio"))]
    #[error("Invalid subscription ID")]
    InvalidSubscriptionId,
    /// Internal channel error
    #[cfg(any(feature = "ws-async-std", feature = "ws-tokio"))]
    #[error("Internal channel error")]
    InternalChannel,
}
