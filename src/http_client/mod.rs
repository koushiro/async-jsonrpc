mod builder;
#[cfg(test)]
mod tests;

use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use jsonrpc_types::*;

pub use self::builder::HttpTransportBuilder;
use crate::{
    error::Result,
    transport::{BatchTransport, Transport},
};

/// HTTP transport
#[derive(Clone)]
pub struct HttpTransport {
    url: String,
    id: Arc<AtomicU64>,
    client: reqwest::Client,
}

impl HttpTransport {
    /// Creates a new HTTP transport with given `url`.
    pub fn new<U: Into<String>>(url: U) -> Result<Self> {
        HttpTransportBuilder::new().build(url)
    }

    /// Creates a `HttpTransportBuilder` to configure a `HttpTransport`.
    ///
    /// This is the same as `HttpTransportBuilder::new()`.
    pub fn builder() -> HttpTransportBuilder {
        HttpTransportBuilder::new()
    }

    async fn request(&self, request: MethodCallRequest) -> Result<Response> {
        let builder = self.client.post(&self.url).json(&request);
        let response = builder.send().await?;
        Ok(response.json().await?)
    }
}

#[async_trait::async_trait]
impl Transport for HttpTransport {
    fn prepare<M: Into<String>>(&self, method: M, params: Option<Params>) -> MethodCall {
        let id = self.id.fetch_add(1, Ordering::AcqRel);
        MethodCall {
            jsonrpc: Version::V2_0,
            method: method.into(),
            params,
            id: Id::Num(id),
        }
    }

    async fn execute(&self, call: MethodCall) -> Result<Response> {
        let request = MethodCallRequest::Single(call);
        self.request(request).await
    }
}

#[async_trait::async_trait]
impl BatchTransport for HttpTransport {
    async fn execute_batch<I>(&self, calls: I) -> Result<Response>
    where
        I: IntoIterator<Item = MethodCall> + Send,
        I::IntoIter: Send,
    {
        let request = MethodCallRequest::Batch(calls.into_iter().collect());
        self.request(request).await
    }
}
