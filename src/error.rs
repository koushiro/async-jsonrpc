use thiserror::Error;

pub(crate) type Result<T, E = RpcError> = std::result::Result<T, E>;

/// The error type for rpc client.
#[derive(Debug, Error)]
pub enum RpcError {
    /// Json serialization/deserialization error.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[cfg(feature = "http-rt-tokio")]
    /// HTTP error.
    #[error(transparent)]
    Http(#[from] reqwest::Error),
    #[cfg(feature = "http-rt-async-std")]
    /// HTTP error.
    #[error(transparent)]
    Http(anyhow::Error),
    /// WebSocket error.
    #[cfg(any(feature = "ws-rt-tokio", feature = "ws-rt-async-std"))]
    #[error(transparent)]
    WebSocket(#[from] async_tungstenite::tungstenite::Error),
    /// Rpc request error, return failure response.
    #[error(transparent)]
    RpcResponse(#[from] jsonrpc_types::Error),
}
