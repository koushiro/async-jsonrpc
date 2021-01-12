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
    #[error("{0}")]
    Http(anyhow::Error),
}
