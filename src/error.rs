use thiserror::Error;

//pub(crate) type Result<T, E = RpcError> = std::result::Result<T, E>;

/// The error type for rpc client.
#[derive(Debug, Error)]
pub enum RpcError {
    /// Json serialization/deserialization error.
    #[error("{0}")]
    Json(#[from] serde_json::Error),
    #[cfg(feature = "http-rt-tokio")]
    /// HTTP error.
    #[error("{0}")]
    Http(#[from] reqwest::Error),
    #[cfg(feature = "http-rt-async-std")]
    /// HTTP error.
    #[error("{0}")]
    Http(anyhow::Error),
    /// WebSocket error.
    #[cfg(any(feature = "ws-rt-tokio", feature = "ws-rt-async-std"))]
    #[error("{0}")]
    WebSocket(#[from] async_tungstenite::tungstenite::Error),
    /// Rpc request error, return failure response.
    #[error("{0}")]
    RpcResponse(#[from] jsonrpc_types::Error),
}
