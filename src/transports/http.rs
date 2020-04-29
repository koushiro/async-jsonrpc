use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::errors::Result;
use crate::transports::{BatchTransport, Transport};
use crate::types::{Call, MethodCall, Params, Request, RequestId, Response, Version};

#[derive(Clone)]
pub struct HttpTransport {
    id: Arc<AtomicUsize>,
    url: String,
    bearer_auth_token: Option<String>,
    client: reqwest::Client,
}

impl HttpTransport {
    fn new_client() -> reqwest::Client {
        reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30))
            .build()
            .expect("ClientBuilder config is valid; qed")
    }

    pub fn new<U: Into<String>>(url: U) -> Self {
        Self {
            id: Default::default(),
            url: url.into(),
            bearer_auth_token: None,
            client: Self::new_client(),
        }
    }

    pub fn new_with_bearer_auth<U: Into<String>, T: Into<String>>(url: U, token: T) -> Self {
        Self {
            id: Default::default(),
            url: url.into(),
            bearer_auth_token: Some(token.into()),
            client: Self::new_client(),
        }
    }

    async fn send_request(&self, request: &Request) -> Result<Response> {
        let builder = self.client.post(&self.url).json(request);
        let builder = if let Some(token) = &self.bearer_auth_token {
            builder.bearer_auth(token)
        } else {
            builder
        };
        Ok(builder.send().await?.json().await?)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Value;

    #[tokio::test]
    async fn test_basic() {
        let http = HttpTransport::new("http://127.0.0.1:1234/rpc/v0");
        // Filecoin.Version need read permission
        let version: Value = http
            .send("Filecoin.Version", Params::Array(vec![]))
            .await
            .unwrap();
        println!("Version: {:?}", version);
    }

    #[tokio::test]
    async fn test_with_bearer_auth() {
        // lotus auth create-token --perm admin
        let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJBbGxvdyI6WyJyZWFkIiwid3JpdGUiLCJzaWduIiwiYWRtaW4iXX0.V82x4rrMmyzgLhW0jeBCL6FVN8I6iSnB0Dc05xeZjVE";
        let http = HttpTransport::new_with_bearer_auth("http://127.0.0.1:1234/rpc/v0", token);
        // Filecoin.LogList need write permission
        let log_list: Value = http
            .send("Filecoin.LogList", Params::Array(vec![]))
            .await
            .unwrap();
        println!("LogList: {:?}", log_list);
    }
}
