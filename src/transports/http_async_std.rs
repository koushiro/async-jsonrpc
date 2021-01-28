use std::{
    fmt,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};

use jsonrpc_types::*;
use surf::http::headers::{self, HeaderName, HeaderValue, HeaderValues, Headers};

use crate::{
    error::Result,
    transports::{BatchTransport, Transport},
};

/// A `HttpTransportBuilder` can be used to create a `HttpTransport` with  custom configuration.
pub struct HttpTransportBuilder {
    headers: Vec<(HeaderName, HeaderValues)>,
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
            headers: Headers::,
            timeout: None,
            connect_timeout: None,
            pool_idle_timeout: Some(Duration::from_secs(90)),
            pool_max_idle_per_host: usize::max_value(),
            tcp_keepalive: None,
            tcp_nodelay: false,
            https_only: false,
        }
    }

    /// Returns a `HttpTransport` that uses this `HttpTransportBuilder` configuration.
    pub fn build<U: Into<String>>(self, url: U) -> Result<HttpTransport> {
        let builder = surf::Client::new()
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
            id: Arc::new(AtomicU64::new(0)),
            client,
        })
    }

    // HTTP header options

    /// Enable HTTP basic authentication.
    pub fn basic_auth<U, P>(self, username: U, password: Option<P>) -> Self
        where
            U: fmt::Display,
            P: fmt::Display,
    {
        let mut basic_auth = b"Basic ".to_vec();
        {
            let mut encoder = base64::write::EncoderWriter::new(&mut basic_auth, base64::STANDARD);
            // The unwraps here are fine because Vec::write* is infallible.
            write!(encoder, "{}:", username).unwrap();
            if let Some(password) = password {
                write!(encoder, "{}", password).unwrap();
            }
        }
        let value = HeaderValue::from_bytes(&basic_auth).expect("HeaderValue::from_bytes()");
        self.header(headers::AUTHORIZATION, value)
    }

    /// Enable HTTP bearer authentication.
    pub fn bearer_auth<T>(self, token: T) -> Self
        where
            T: fmt::Display,
    {
        let bearer_auth = format!("Bearer {}", token);
        let value = HeaderValue::from_str(&bearer_auth).expect("HeaderValue::from_str()");
        self.header(headers::AUTHORIZATION, value)
    }

    /// Adds a `Header` for every request.
    pub fn header(mut self, name: HeaderName, value: HeaderValue) -> Self {
        self.headers.insert(name, value);
        self
    }

    /// Adds `Header`s for every request.
    pub fn headers(mut self, headers: HeaderMap) -> Self {
        for (key, value) in headers.iter() {
            self.headers.insert(key, value.clone());
        }
        self
    }

    // Timeout options

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

    // TLS options

    /// Restrict the Client to be used with HTTPS only requests.
    ///
    /// Defaults to false.
    pub fn https_only(mut self, enabled: bool) -> Self {
        self.https_only = enabled;
        self
    }
}

/// HTTP transport
#[derive(Clone)]
pub struct HttpTransport {
    url: String,
    id: Arc<AtomicU64>,
    client: surf::Client,
}

impl HttpTransport {
    /// Creates a new HTTP transport with given `url`.
    pub fn new<U: Into<String>>(url: U) -> Self {
        HttpTransportBuilder::new()
            .build(url)
            .expect("Client::new()")
    }

    /// Creates a `HttpTransportBuilder` to configure a `HttpTransport`.
    ///
    /// This is the same as `HttpTransportBuilder::new()`.
    pub fn builder() -> HttpTransportBuilder {
        HttpTransportBuilder::new()
    }

    async fn send_request(&self, request: Request) -> Result<Response> {
        let request = serde_json::to_string(&request)?;
        let builder = surf::post(&self.url)
            .content_type(surf::http::mime::JSON)
            .body(request);
        let mut response = builder.send().await?;
        Ok(response.body_json().await?)
    }
}

#[async_trait::async_trait]
impl Transport for HttpTransport {
    fn prepare<M: Into<String>>(&self, method: M, params: Option<Params>) -> MethodCall {
        let id = self.id.fetch_add(1, Ordering::AcqRel);
        MethodCall {
            jsonrpc: Some(Version::V2_0),
            method: method.into(),
            params,
            id: Id::Num(id),
        }
    }

    async fn execute(&self, request: Request) -> Result<Response> {
        self.send_request(request).await
    }
}
