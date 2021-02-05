use std::{fmt, time::Duration};

use async_tungstenite::tungstenite::handshake::client::Request as HandShakeRequest;
use futures::channel::mpsc;
use http::header::{self, HeaderMap, HeaderName, HeaderValue};

use crate::{
    error::WsError,
    ws_client::{task::WsTask, WsClient},
};

/// A `WsClientBuilder` can be used to create a `HttpClient` with  custom configuration.
#[derive(Debug)]
pub struct WsClientBuilder {
    headers: HeaderMap,
    timeout: Option<Duration>,
    max_concurrent_request_capacity: usize,
    max_capacity_per_subscription: usize,
}

impl Default for WsClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl WsClientBuilder {
    /// Creates a new `WsClientBuilder`.
    ///
    /// This is the same as `WsClient::builder()`.
    pub fn new() -> Self {
        Self {
            headers: HeaderMap::new(),
            timeout: None,
            max_concurrent_request_capacity: 256,
            max_capacity_per_subscription: 64,
        }
    }

    // ========================================================================
    // HTTP header options
    // ========================================================================

    /// Enables basic authentication.
    pub fn basic_auth<U, P>(self, username: U, password: Option<P>) -> Self
    where
        U: fmt::Display,
        P: fmt::Display,
    {
        let mut basic_auth = "Basic ".to_string();
        let auth = if let Some(password) = password {
            base64::encode(format!("{}:{}", username, password))
        } else {
            base64::encode(format!("{}:", username))
        };
        basic_auth.push_str(&auth);
        let value = HeaderValue::from_str(&basic_auth).expect("basic auth header value");
        self.header(header::AUTHORIZATION, value)
    }

    /// Enables bearer authentication.
    pub fn bearer_auth<T>(self, token: T) -> Self
    where
        T: fmt::Display,
    {
        let bearer_auth = format!("Bearer {}", token);
        let value = HeaderValue::from_str(&bearer_auth).expect("bearer auth header value");
        self.header(header::AUTHORIZATION, value)
    }

    /// Adds a `Header` for handshake request.
    pub fn header(mut self, name: HeaderName, value: HeaderValue) -> Self {
        self.headers.insert(name, value);
        self
    }

    /// Adds `Header`s for handshake request.
    pub fn headers(mut self, headers: HeaderMap) -> Self {
        self.headers.extend(headers);
        self
    }

    // ========================================================================
    // Channel options
    // ========================================================================

    /// Sets the max channel capacity of sending request concurrently.
    ///
    /// Default is 256.
    pub fn max_concurrent_request_capacity(mut self, capacity: usize) -> Self {
        self.max_concurrent_request_capacity = capacity;
        self
    }

    /// Sets the max channel capacity of every subscription stream.
    ///
    /// Default is 64.
    pub fn max_capacity_per_subscription(mut self, capacity: usize) -> Self {
        self.max_capacity_per_subscription = capacity;
        self
    }

    // ========================================================================
    // Timeout options
    // ========================================================================

    /// Enables a request timeout.
    ///
    /// The timeout is applied from when the request starts connecting until the
    /// response body has finished.
    ///
    /// Default is no timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    // ========================================================================

    /// Returns a `WsClient` that uses this `WsClientBuilder` configuration.
    pub async fn build(self, url: impl Into<String>) -> Result<WsClient, WsError> {
        let url = url.into();
        let mut handshake_builder = HandShakeRequest::get(&url);
        let headers = handshake_builder.headers_mut().expect("handshake request just created");
        headers.extend(self.headers);
        let handshake_req = handshake_builder.body(()).map_err(WsError::HttpFormat)?;

        let (to_back, from_front) = mpsc::channel(self.max_concurrent_request_capacity);
        let task = WsTask::handshake(handshake_req, self.max_capacity_per_subscription).await?;
        #[cfg(feature = "ws-async-std")]
        let _handle = async_std::task::spawn(task.into_task(from_front));
        #[cfg(feature = "ws-tokio")]
        let _handle = tokio::spawn(task.into_task(from_front));

        Ok(WsClient {
            to_back,
            timeout: self.timeout,
        })
    }
}
