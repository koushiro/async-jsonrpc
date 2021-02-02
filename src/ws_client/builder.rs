use std::{fmt, time::Duration};

use async_tungstenite::tungstenite::handshake::client::Request as HandShakeRequest;
use futures::channel::mpsc;
use http::header::{self, HeaderMap, HeaderName, HeaderValue};

use crate::{
    error::Result,
    ws_client::{task::WsTask, WsTransport},
};

/// A `WsTransportBuilder` can be used to create a `HttpTransport` with  custom configuration.
#[derive(Debug)]
pub struct WsTransportBuilder {
    headers: HeaderMap,
    timeout: Option<Duration>,
    connection_timeout: Option<Duration>,
}

impl Default for WsTransportBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl WsTransportBuilder {
    /// Creates a new `WsTransportBuilder`.
    ///
    /// This is the same as `WsTransport::builder()`.
    pub fn new() -> Self {
        Self {
            headers: HeaderMap::new(),
            timeout: None,
            connection_timeout: None,
        }
    }

    // ========================================================================
    // HTTP header options
    // ========================================================================

    /// Enable basic authentication.
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

    /// Enable bearer authentication.
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

    /// Set a timeout for only the connect phase of a `Client`.
    ///
    /// Default is `None`.
    ///
    /// # Note
    ///
    /// This **requires** the futures be executed in a tokio runtime with
    /// a tokio timer enabled.
    pub fn connection_timeout(mut self, timeout: Duration) -> Self {
        self.connection_timeout = Some(timeout);
        self
    }

    // ========================================================================

    /// Returns a `WsTransport` that uses this `WsTransportBuilder` configuration.
    pub async fn build(self, url: impl Into<String>) -> Result<WsTransport> {
        let url = url.into();
        let mut handshake_builder = HandShakeRequest::get(&url);
        let headers = handshake_builder
            .headers_mut()
            .expect("HandShakeRequest just created");
        headers.extend(self.headers);
        let handshake_req = handshake_builder.body(())?;

        let task = WsTask::handshake(handshake_req).await?;

        let (to_back, from_front) = mpsc::channel(256);
        #[cfg(feature = "ws-async-std")]
        let _handle = async_std::task::spawn(task.into_task(from_front));
        #[cfg(feature = "ws-tokio")]
        let _handle = tokio::spawn(task.into_task(from_front));

        Ok(WsTransport { to_back })
    }
}
