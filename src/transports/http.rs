use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::errors::Result;
use crate::transports::{BatchTransport, Transport};
use crate::types::{Call, MethodCall, Params, Request, RequestId, Response, Version};

/// HTTP transport
#[derive(Clone)]
pub struct HttpTransport {
    id: Arc<AtomicUsize>,
    url: String,
    bearer_auth_token: Option<String>,
    client: surf::Client,
}

impl HttpTransport {
    fn new_client() -> surf::Client {
        surf::Client::new()
    }

    /// Create a new HTTP transport with given `url`.
    pub fn new<U: Into<String>>(url: U) -> Self {
        Self {
            id: Default::default(),
            url: url.into(),
            bearer_auth_token: None,
            client: Self::new_client(),
        }
    }

    /// Create a new HTTP transport with given `url` and bearer `token`.
    pub fn new_with_bearer_auth<U: Into<String>, T: Into<String>>(url: U, token: T) -> Self {
        Self {
            id: Default::default(),
            url: url.into(),
            bearer_auth_token: Some(token.into()),
            client: Self::new_client(),
        }
    }

    async fn send_request(&self, request: &Request) -> Result<Response> {
        let builder = surf::post(&self.url)
            .content_type(surf::http::mime::JSON)
            .body(
                surf::Body::from_json(request)
                    .map_err(|err| crate::RpcError::Http(err.into_inner()))?,
            );

        // let builder = self.client.post(&self.url).json(request);
        let builder = if let Some(token) = &self.bearer_auth_token {
            builder.header(
                surf::http::headers::AUTHORIZATION,
                format!("Bearer {}", token),
            )
        } else {
            builder
        };

        Ok(builder
            .recv_json()
            .await
            .map_err(|err| crate::RpcError::Http(err.into_inner()))?)
    }
}

#[async_trait::async_trait]
impl Transport for HttpTransport {
    fn prepare<M: Into<String>>(&self, method: M, params: Params) -> (RequestId, Call) {
        let id = self.id.fetch_add(1, Ordering::AcqRel);
        let call = Call::MethodCall(MethodCall {
            jsonrpc: Some(Version::V2),
            id,
            method: method.into(),
            params,
        });
        (id, call)
    }

    async fn execute(&self, _id: RequestId, request: &Request) -> Result<Response> {
        self.send_request(request).await
    }
}

#[async_trait::async_trait]
impl BatchTransport for HttpTransport {}
