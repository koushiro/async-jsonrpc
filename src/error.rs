use thiserror::Error;

pub(crate) type Result<T, E = ClientError> = std::result::Result<T, E>;

/// WebSocket error type.
#[cfg(any(feature = "ws-tokio", feature = "ws-async-std"))]
pub use async_tungstenite::tungstenite::Error as WsError;

/// The error type for rpc transport.
#[derive(Debug, Error)]
pub enum ClientError {
    /// Json serialization/deserialization error.
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    /// HTTP error.
    #[cfg(feature = "http-tokio")]
    #[error(transparent)]
    Http(#[from] reqwest::Error),

    /// HTTP error.
    #[cfg(feature = "http-async-std")]
    #[error(transparent)]
    Http(anyhow::Error),

    /// WebSocket protocol error.
    #[cfg(any(feature = "ws-tokio", feature = "ws-async-std"))]
    #[error(transparent)]
    WebSocket(#[from] WsError),
    /// WebSocket request timeout.
    #[cfg(any(feature = "ws-tokio", feature = "ws-async-std"))]
    #[error("WebSocket request timeout")]
    WsRequestTimeout,
    /// Duplicate request ID.
    #[cfg(any(feature = "ws-tokio", feature = "ws-async-std"))]
    #[error("Duplicate request ID")]
    DuplicateRequestId,
    /// Invalid Request ID.
    #[cfg(any(feature = "ws-tokio", feature = "ws-async-std"))]
    #[error("Invalid request ID")]
    InvalidRequestId,
    /// Invalid Subscription ID.
    #[cfg(any(feature = "ws-tokio", feature = "ws-async-std"))]
    #[error("Invalid subscription ID")]
    InvalidSubscriptionId,
    /*
    /// Internal task finished
    #[cfg(any(feature = "ws-tokio", feature = "ws-async-std"))]
    #[error("Cannot send request, internal task finished")]
    InternalTaskFinish,
    */
    /// Internal channel error
    #[cfg(any(feature = "ws-tokio", feature = "ws-async-std"))]
    #[error("Internal channel error")]
    InternalChannel,
}
