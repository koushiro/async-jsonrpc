mod builder;
#[cfg(test)]
mod tests;

use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use jsonrpc_types::v2::*;
use serde::{de::DeserializeOwned, Serialize};

pub use self::builder::HttpClientBuilder;
use crate::{
    error::HttpClientError,
    transport::{BatchTransport, Transport},
};

/// HTTP JSON-RPC client
#[cfg(feature = "http-async-std")]
#[derive(Clone)]
pub struct HttpClient {
    url: String,
    id: Arc<AtomicU64>,
    client: surf::Client,
    headers: http::header::HeaderMap,
    timeout: Option<std::time::Duration>,
}

/// HTTP JSON-RPC client
#[cfg(feature = "http-tokio")]
#[derive(Clone)]
pub struct HttpClient {
    url: String,
    id: Arc<AtomicU64>,
    client: reqwest::Client,
}

impl HttpClient {
    /// Creates a new HTTP JSON-RPC client with given `url`.
    pub fn new<U: Into<String>>(url: U) -> Result<Self, HttpClientError> {
        HttpClientBuilder::new().build(url)
    }

    /// Creates a `HttpClientBuilder` to configure a `HttpClient`.
    ///
    /// This is the same as `HttpClientBuilder::new()`.
    pub fn builder() -> HttpClientBuilder {
        HttpClientBuilder::new()
    }
}

#[cfg(feature = "http-async-std")]
impl HttpClient {
    async fn send_request<REQ, RSP>(&self, request: REQ) -> Result<RSP, HttpClientError>
    where
        REQ: Serialize,
        RSP: Serialize + DeserializeOwned,
    {
        let request = serde_json::to_string(&request).expect("serialize request");
        log::debug!("Request: {}", request);

        let mut builder = self
            .client
            .post(&self.url)
            .content_type(surf::http::mime::JSON)
            .body(request);
        for (header_name, header_value) in self.headers.iter() {
            builder = builder.header(
                header_name.as_str(),
                header_value.to_str().expect("must be visible ascii"),
            );
        }

        let response = builder.send();
        let response = if let Some(duration) = self.timeout {
            let timeout = async_std::task::sleep(duration);
            futures::pin_mut!(response, timeout);
            match futures::future::select(response, timeout).await {
                futures::future::Either::Left((response, _)) => response,
                futures::future::Either::Right((_, _)) => return Err(anyhow::anyhow!("http request timeout").into()),
            }
        } else {
            response.await
        };
        let mut response = response.map_err(|err| err.into_inner())?;

        let response = response.body_string().await.map_err(|err| err.into_inner())?;
        log::debug!("Response: {}", response);

        Ok(serde_json::from_str::<RSP>(&response)?)
    }
}

#[cfg(feature = "http-tokio")]
impl HttpClient {
    async fn send_request<REQ, RSP>(&self, request: REQ) -> Result<RSP, HttpClientError>
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
    type Error = HttpClientError;

    async fn request<M>(&self, method: M, params: Option<Params>) -> Result<Output, Self::Error>
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
    async fn request_batch<I, M>(&self, batch: I) -> Result<Vec<Output>, <Self as Transport>::Error>
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
