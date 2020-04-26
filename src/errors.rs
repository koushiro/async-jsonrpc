pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Http(#[from] reqwest::Error),
    #[error("{0}")]
    WebSocket(#[from] async_tungstenite::tungstenite::Error),
    #[error("{0}")]
    Rpc(#[from] crate::types::Error),
}
