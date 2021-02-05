use futures::stream::Stream;
use jsonrpc_types::*;

use crate::error::Result;

/// A JSON-RPC 2.0 transport.
#[async_trait::async_trait]
pub trait Transport {
    /// Send a RPC call with the given method and parameters.
    async fn request<M>(&self, method: M, params: Option<Params>) -> Result<Output>
    where
        M: Into<String> + Send;
}

/// A JSON-RPC 2.0 transport supporting batch requests.
#[async_trait::async_trait]
pub trait BatchTransport: Transport {
    /// Send a batch of RPC calls with the given method and parameters.
    async fn request_batch<I, M>(&self, batch: I) -> Result<Vec<Output>>
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

    /// Add a subscription to this transport
    async fn subscribe<M>(
        &self,
        subscribe_method: M,
        unsubscribe_method: M,
        params: Option<Params>,
    ) -> Result<(Id, Self::NotificationStream)>
    where
        M: Into<String> + Send;

    /// Remove a subscription from this transport
    async fn unsubscribe<M>(&self, unsubscribe_method: M, subscription_id: Id) -> Result<bool>
    where
        M: Into<String> + Send;
}
