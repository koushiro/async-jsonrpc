use std::error::Error;

use futures::stream::Stream;
use jsonrpc_types::*;

/// A JSON-RPC 2.0 transport.
#[async_trait::async_trait]
pub trait Transport {
    /// The transport error type.
    type Error: Error;

    /// Send a RPC call with the given method and parameters.
    async fn request<M>(&self, method: M, params: Option<Params>) -> Result<Output, Self::Error>
    where
        M: Into<String> + Send;
}

/// A JSON-RPC 2.0 transport supporting batch requests.
#[async_trait::async_trait]
pub trait BatchTransport: Transport {
    /// Send a batch of RPC calls with the given method and parameters.
    async fn request_batch<I, M>(&self, batch: I) -> Result<Vec<Output>, Self::Error>
    where
        I: IntoIterator<Item = (M, Option<Params>)> + Send,
        I::IntoIter: Send,
        M: Into<String>;
}

/// A JSON-RPC 2.0 transport supporting subscriptions.
#[async_trait::async_trait]
pub trait PubsubTransport: Transport {
    /// The subscription stream.
    type NotificationStream: Stream<Item = SubscriptionNotification>;

    /// Add a subscription to this transport.
    ///
    /// Will send unsubscribe request to the server when drop the notification stream.
    async fn subscribe<M>(
        &self,
        subscribe_method: M,
        params: Option<Params>,
    ) -> Result<(Id, Self::NotificationStream), Self::Error>
    where
        M: Into<String> + Send;

    /// Send an unsubscribe request to the server manually.
    async fn unsubscribe<M>(&self, unsubscribe_method: M, subscription_id: Id) -> Result<bool, Self::Error>
    where
        M: Into<String> + Send;
}
