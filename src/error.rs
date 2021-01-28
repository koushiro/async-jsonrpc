use thiserror::Error;

pub(crate) type Result<T, E = RpcClientError> = std::result::Result<T, E>;

/// The error type for rpc client.
#[derive(Debug, Error)]
pub enum RpcClientError {
    /// Json serialization/deserialization error.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[cfg(feature = "http-tokio")]
    /// HTTP request error.
    #[error(transparent)]
    HttpRequest(#[from] reqwest::Error),
    #[cfg(feature = "http-async-std")]
    /// HTTP request error.
    #[error(transparent)]
    HttpRequest(anyhow::Error),
    /// HTTP connection error.
    #[cfg(any(feature = "ws-tokio", feature = "ws-async-std"))]
    #[error(transparent)]
    HttpConnection(#[from] async_tungstenite::tungstenite::http::Error),
    /// WebSocket protocol error.
    #[cfg(any(feature = "ws-tokio", feature = "ws-async-std"))]
    #[error(transparent)]
    WebSocket(#[from] async_tungstenite::tungstenite::Error),
    /// Internal task finished
    #[cfg(any(feature = "ws-tokio", feature = "ws-async-std"))]
    #[error("Cannot send request, internal task finished")]
    InternalTaskFinish,

    /// Rpc request error, return failure response.
    #[error(transparent)]
    RpcResponse(#[from] jsonrpc_types::Error),
}
