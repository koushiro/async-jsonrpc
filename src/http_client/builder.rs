use std::{
    fmt,
    sync::{atomic::AtomicU64, Arc},
    time::Duration,
};

use http::header::{self, HeaderMap, HeaderName, HeaderValue};

use crate::{error::Result, http_client::HttpTransport};

/// A `HttpTransportBuilder` can be used to create a `HttpTransport` with  custom configuration.
#[derive(Debug)]
pub struct HttpTransportBuilder {
    headers: HeaderMap,
    timeout: Option<Duration>,
    connect_timeout: Option<Duration>,
    pool_idle_timeout: Option<Duration>,
    pool_max_idle_per_host: usize,
    tcp_keepalive: Option<Duration>,
    tcp_nodelay: bool,
    https_only: bool,
}

impl Default for HttpTransportBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpTransportBuilder {
    /// Creates a new `HttpTransportBuilder`.
    ///
    /// This is the same as `HttpTransport::builder()`.
    pub fn new() -> Self {
        Self {
            headers: HeaderMap::new(),
            timeout: None,
            connect_timeout: None,
            pool_idle_timeout: Some(Duration::from_secs(90)),
            pool_max_idle_per_host: usize::max_value(),
            tcp_keepalive: None,
            tcp_nodelay: false,
            https_only: false,
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
    ///
    /// # Note
    ///
    /// This **requires** the futures be executed in a tokio runtime with
    /// a tokio timer enabled.
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = Some(timeout);
        self
    }

    /// Set an optional timeout for idle sockets being kept-alive.
    ///
    /// Pass `None` to disable timeout.
    ///
    /// Default is 90 seconds.
    pub fn pool_idle_timeout(mut self, val: Duration) -> Self {
        self.pool_idle_timeout = Some(val);
        self
    }

    /// Sets the maximum idle connection per host allowed in the pool.
    pub fn pool_max_idle_per_host(mut self, max: usize) -> Self {
        self.pool_max_idle_per_host = max;
        self
    }

    // TCP options

    /// Set whether sockets have `SO_NODELAY` enabled.
    ///
    /// Default is `true`.
    pub fn tcp_nodelay(mut self, enabled: bool) -> Self {
        self.tcp_nodelay = enabled;
        self
    }

    /// Set that all sockets have `SO_KEEPALIVE` set with the supplied duration.
    ///
    /// If `None`, the option will not be set.
    pub fn tcp_keepalive(mut self, val: Duration) -> Self {
        self.tcp_keepalive = Some(val);
        self
    }

    // ========================================================================
    // TLS options
    // ========================================================================

    /// Restrict the Client to be used with HTTPS only requests.
    ///
    /// Defaults to false.
    pub fn https_only(mut self, enabled: bool) -> Self {
        self.https_only = enabled;
        self
    }

    // ========================================================================

    /// Returns a `HttpTransport` that uses this `HttpTransportBuilder` configuration.
    pub fn build<U: Into<String>>(self, url: U) -> Result<HttpTransport> {
        let builder = reqwest::Client::builder()
            .default_headers(self.headers)
            .pool_idle_timeout(self.pool_idle_timeout)
            .pool_max_idle_per_host(self.pool_max_idle_per_host)
            .tcp_keepalive(self.tcp_keepalive)
            .tcp_nodelay(self.tcp_nodelay)
            .https_only(self.https_only);
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
        Ok(HttpTransport {
            url: url.into(),
            id: Arc::new(AtomicU64::new(1)),
            client,
        })
    }
}
