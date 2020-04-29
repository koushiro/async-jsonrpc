pub type Result<T> = std::result::Result<T, RpcError>;

#[derive(Debug, thiserror::Error)]
pub enum RpcError {
    #[error("{0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Http(#[from] reqwest::Error),
    #[error("{0}")]
    WebSocket(#[from] async_tungstenite::tungstenite::Error),
    #[error("{0}")]
    RpcResponse(#[from] crate::types::Error),
}
