use std::{
    fmt,
    sync::{atomic::AtomicU64, Arc},
    time::Duration,
};

use http::header::{self, HeaderMap, HeaderName, HeaderValue};

use crate::{error::Result, http_client::HttpClient};

/// A `HttpClientBuilder` can be used to create a `HttpClient` with  custom configuration.
#[derive(Debug)]
pub struct HttpClientBuilder {
    pub(crate) headers: HeaderMap,
    timeout: Option<Duration>,
    connect_timeout: Option<Duration>,
}

impl Default for HttpClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpClientBuilder {
    /// Creates a new `HttpClientBuilder`.
    ///
    /// This is the same as `HttpClient::builder()`.
    pub fn new() -> Self {
        Self {
            headers: HeaderMap::new(),
            timeout: None,
            connect_timeout: None,
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

    /// Adds a `Header` for every request.
    pub fn header(mut self, name: HeaderName, value: HeaderValue) -> Self {
        self.headers.insert(name, value);
        self
    }

    /// Adds `Header`s for every request.
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
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = Some(timeout);
        self
    }

    // ========================================================================

    /// Returns a `HttpClient` that uses this `HttpClientBuilder` configuration.
    pub fn build<U: Into<String>>(self, url: U) -> Result<HttpClient> {
        let builder = reqwest::Client::builder().default_headers(self.headers);
        let builder = if let Some(timeout) = self.timeout {
            builder.timeout(timeout)
        } else {
            builder
        };
        let builder = if let Some(timeout) = self.connect_timeout {
            builder.connect_timeout(timeout)
        } else {
            builder
        };
        let client = builder.build()?;
        Ok(HttpClient {
            url: url.into(),
            id: Arc::new(AtomicU64::new(1)),
            client,
        })
    }
}
