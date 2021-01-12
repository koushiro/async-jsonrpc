/// A result type that wraps up the rpc client errors.
pub type Result<T, E = RpcError> = std::result::Result<T, E>;

/// The error type for rpc client.
#[derive(Debug, thiserror::Error)]
pub enum RpcError {
    /// Json serialization/deserialization error.
    #[error("{0}")]
    Json(#[from] serde_json::Error),
    /// Rpc request error, return failure response.
    #[error("{0}")]
    RpcResponse(#[from] crate::types::Error),
    /// HTTP error.
    #[cfg(feature = "http")]
    #[error("{0}")]
    Http(#[from] reqwest::Error),
    #[cfg(feature = "http-surf")]
    /// HTTP error (surf).
    #[error("{0}")]
    Http(anyhow::Error),
    /// WebSocket error.
    #[cfg(feature = "ws")]
    #[error("{0}")]
    WebSocket(#[from] async_tungstenite::tungstenite::Error),
}
