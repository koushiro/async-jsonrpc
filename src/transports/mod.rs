// #[cfg(feature = "http-rt-async-std")]
// mod http_async_std;
// #[cfg(feature = "http-rt-async-std")]
// pub use self::http_async_std::*;

// mod http_tokio;
// #[cfg(feature = "http-rt-tokio")]
// pub use self::http_tokio::*;

#[cfg(feature = "http-tokio")]
//#[cfg(any(feature = "http-async-std", feature = "http-tokio"))]
mod http;
#[cfg(feature = "http-tokio")]
//#[cfg(any(feature = "http-async-std", feature = "http-tokio"))]
pub use self::http::{HttpTransport, HttpTransportBuilder};

// #[cfg(feature = "ws-tokio")]
// #[cfg(any(feature = "ws-async-std", feature = "ws-tokio"))]
mod ws;
#[cfg(feature = "ws-tokio")]
// #[cfg(any(feature = "ws-async-std", feature = "ws-tokio"))]
pub use self::ws::{WsTransport, WsTransportBuilder};

use futures::stream::Stream;
use jsonrpc_types::*;

use crate::error::Result;

/// A transport implementation.
#[async_trait::async_trait]
pub trait Transport {
    /// Prepare serializable RPC call for given method with parameters.
    fn prepare<M: Into<String>>(&self, method: M, params: Option<Params>) -> MethodCall;

    /// Execute prepared RPC call.
    async fn execute(&self, request: MethodCallRequest) -> Result<Response>;

    /// Send a RPC call with the given method and parameters.
    async fn send<M>(&self, method: M, params: Option<Params>) -> Result<Response>
    where
        M: Into<String> + Send,
    {
        let call = self.prepare(method, params);
        let request = MethodCallRequest::Single(call);
        log::debug!(
            "Request: {}",
            serde_json::to_string(&request)
                .expect("Serialize `MethodCallRequest` shouldn't be failed")
        );

        let response = self.execute(request).await?;
        log::debug!(
            "Response: {}",
            serde_json::to_string(&response).expect("Serialize `Response` shouldn't be failed")
        );
        Ok(response)
    }
}

/// A transport implementation supporting batch requests
#[async_trait::async_trait]
pub trait BatchTransport: Transport {
    /// Send a batch of RPC calls with the given method and parameters.
    async fn send_batch<I, M>(&self, batch: I) -> Result<Response>
    where
        I: IntoIterator<Item = (M, Option<Params>)> + Send,
        I::IntoIter: Send,
        M: Into<String>,
    {
        let calls = batch
            .into_iter()
            .map(|(method, params)| self.prepare(method, params))
            .collect::<Vec<_>>();
        let request = MethodCallRequest::Batch(calls);
        log::debug!(
            "Request: {}",
            serde_json::to_string(&request)
                .expect("Serialize `MethodCallRequest` shouldn't be failed")
        );

        let response = self.execute(request).await?;
        log::debug!(
            "Response: {}",
            serde_json::to_string(&response).expect("Serialize `Response` shouldn't be failed")
        );
        Ok(response)
    }
}

/// A transport implementation supporting pub sub subscriptions.
pub trait PubsubTransport: Transport {
    /// The type of stream this transport returns
    type NotificationStream: Stream<Item = Notification>;

    /// Add a subscription to this transport
    fn subscribe<T>(&self, id: Id) -> Result<Self::NotificationStream>;

    /// Remove a subscription from this transport
    fn unsubscribe(&self, id: Id) -> Result<()>;
}
