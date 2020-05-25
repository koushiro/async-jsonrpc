/// A result type that wraps up the rpc client errors.
pub type Result<T> = std::result::Result<T, RpcError>;

/// The error type for rpc client.
#[derive(Debug, thiserror::Error)]
pub enum RpcError {
    /// Json serialization/deserialization error.
    #[error("{0}")]
    Json(#[from] serde_json::Error),
    /// HTTP error.
    #[error("{0}")]
    Http(#[from] reqwest::Error),
    /// WebSocket error.
    #[error("{0}")]
    WebSocket(#[from] async_tungstenite::tungstenite::Error),
    /// Rpc request error, return failure response.
    #[error("{0}")]
    RpcResponse(#[from] crate::types::Error),
}
