mod builder;
#[cfg(test)]
mod tests;

use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use jsonrpc_types::*;
use serde::{de::DeserializeOwned, Serialize};

pub use self::builder::HttpClientBuilder;
use crate::{
    error::Result,
    transport::{BatchTransport, Transport},
};

/// HTTP transport
#[derive(Clone)]
pub struct HttpClient {
    url: String,
    id: Arc<AtomicU64>,
    client: reqwest::Client,
}

impl HttpClient {
    /// Creates a new HTTP transport with given `url`.
    pub fn new<U: Into<String>>(url: U) -> Result<Self> {
        HttpClientBuilder::new().build(url)
    }

    /// Creates a `HttpTransportBuilder` to configure a `HttpTransport`.
    ///
    /// This is the same as `HttpTransportBuilder::new()`.
    pub fn builder() -> HttpClientBuilder {
        HttpClientBuilder::new()
    }

    async fn send_request<REQ, RSP>(&self, request: REQ) -> Result<RSP>
    where
        REQ: Serialize,
        RSP: Serialize + DeserializeOwned,
    {
        log::debug!(
            "Request: {}",
            serde_json::to_string(&request).expect("serialize request")
        );
        let builder = self.client.post(&self.url).json(&request);
        let response = builder.send().await?;
        let response = response.json().await?;
        log::debug!(
            "Response: {}",
            serde_json::to_string(&response).expect("serialize response")
        );
        Ok(response)
    }
}

#[async_trait::async_trait]
impl Transport for HttpClient {
    async fn request<M>(&self, method: M, params: Option<Params>) -> Result<Output>
    where
        M: Into<String> + Send,
    {
        let id = self.id.fetch_add(1, Ordering::AcqRel);
        let call = MethodCall::new(method, params, Id::Num(id));
        self.send_request(call).await
    }
}

#[async_trait::async_trait]
impl BatchTransport for HttpClient {
    async fn request_batch<I, M>(&self, batch: I) -> Result<Vec<Output>>
    where
        I: IntoIterator<Item = (M, Option<Params>)> + Send,
        I::IntoIter: Send,
        M: Into<String>,
    {
        let calls = batch
            .into_iter()
            .map(|(method, params)| {
                let id = self.id.fetch_add(1, Ordering::AcqRel);
                MethodCall::new(method, params, Id::Num(id))
            })
            .collect::<Vec<_>>();
        self.send_request(calls).await
    }
}
