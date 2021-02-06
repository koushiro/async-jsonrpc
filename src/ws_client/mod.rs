mod builder;
mod manager;
mod task;
#[cfg(test)]
mod tests;

use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use futures::{
    channel::{mpsc, oneshot},
    future::{self, FutureExt},
    sink::SinkExt,
    stream::{Stream, StreamExt},
};
use jsonrpc_types::*;

pub use self::builder::WsClientBuilder;
use crate::{
    error::{ClientError, Result},
    transport::{BatchTransport, PubsubTransport, Transport},
};

/// Message that the client can send to the background task.
pub(crate) enum ToBackTaskMessage {
    Request {
        method: String,
        params: Option<Params>,
        /// One-shot channel where to send back the response of the request.
        send_back: oneshot::Sender<Result<Output>>,
    },
    BatchRequest {
        batch: Vec<(String, Option<Params>)>,
        /// One-shot channel where to send back the response of the batch request.
        send_back: oneshot::Sender<Result<Vec<Output>>>,
    },
    Subscribe {
        subscribe_method: String,
        unsubscribe_method: String,
        params: Option<Params>,
        /// One-shot channel where to send back the response (subscription id) and a `Receiver`
        /// that will receive subscription notification when we get a response (subscription id)
        /// from the server about the subscription.
        send_back: oneshot::Sender<Result<(Id, mpsc::Receiver<SubscriptionNotification>)>>,
    },
    /// When a subscription channel is closed, we send this message to the backend task to clean
    /// the subscription.
    SubscriptionClosed(Id),
}

/// WebSocket JSON-RPC client
#[derive(Clone)]
pub struct WsClient {
    to_back: mpsc::Sender<ToBackTaskMessage>,
    /// Request timeout.
    timeout: Option<Duration>,
}

impl WsClient {
    /// Creates a new WebSocket JSON-RPC client.
    pub async fn new(url: impl Into<String>) -> Result<Self> {
        WsClientBuilder::new().build(url).await.map_err(ClientError::WebSocket)
    }

    /// Creates a `WsClientBuilder` to configure a `WsClient`.
    ///
    /// This is the same as `WsClientBuilder::new()`.
    pub fn builder() -> WsClientBuilder {
        WsClientBuilder::new()
    }

    /// Sends a `method call` request to the server.
    async fn send_request(&self, method: impl Into<String>, params: Option<Params>) -> Result<Output> {
        let method = method.into();
        log::debug!("[frontend] Send request: method={}, params={:?}", method, params);

        let (tx, rx) = oneshot::channel();
        self.to_back
            .clone()
            .send(ToBackTaskMessage::Request {
                method,
                params,
                send_back: tx,
            })
            .await
            .map_err(|_| ClientError::InternalChannel)?;

        let res = if let Some(duration) = self.timeout {
            #[cfg(feature = "ws-async-std")]
            let timeout = async_std::task::sleep(duration);
            #[cfg(feature = "ws-tokio")]
            let timeout = tokio::time::sleep(duration);
            futures::pin_mut!(rx, timeout);
            match future::select(rx, timeout).await {
                future::Either::Left((response, _)) => response,
                future::Either::Right((_, _)) => return Err(ClientError::WsRequestTimeout),
            }
        } else {
            rx.await
        };
        match res {
            Ok(Ok(output)) => Ok(output),
            Ok(Err(err)) => Err(err),
            Err(_) => Err(ClientError::InternalChannel),
        }
    }

    /// Sends a batch of `method call` requests to the server.
    async fn send_request_batch<I, M>(&self, batch: I) -> Result<Vec<Output>>
    where
        I: IntoIterator<Item = (M, Option<Params>)>,
        M: Into<String>,
    {
        let batch = batch
            .into_iter()
            .map(|(method, params)| (method.into(), params))
            .collect::<Vec<_>>();
        log::debug!("[frontend] Send a batch of requests: {:?}", batch);

        let (tx, rx) = oneshot::channel();
        self.to_back
            .clone()
            .send(ToBackTaskMessage::BatchRequest { batch, send_back: tx })
            .await
            .map_err(|_| ClientError::InternalChannel)?;

        let res = if let Some(duration) = self.timeout {
            #[cfg(feature = "ws-async-std")]
            let timeout = async_std::task::sleep(duration);
            #[cfg(feature = "ws-tokio")]
            let timeout = tokio::time::sleep(duration);
            futures::pin_mut!(rx, timeout);
            match future::select(rx, timeout).await {
                future::Either::Left((response, _)) => response,
                future::Either::Right((_, _)) => return Err(ClientError::WsRequestTimeout),
            }
        } else {
            rx.await
        };
        match res {
            Ok(Ok(outputs)) => Ok(outputs),
            Ok(Err(err)) => Err(err),
            Err(_) => Err(ClientError::InternalChannel),
        }
    }

    /// Sends a subscribe request to the server.
    ///
    /// `subscribe_method` and `params` are used to ask for the subscription towards the server.
    /// `unsubscribe_method` is used to close the subscription.
    async fn send_subscribe(
        &self,
        subscribe_method: impl Into<String>,
        unsubscribe_method: impl Into<String>,
        params: Option<Params>,
    ) -> Result<WsSubscription<SubscriptionNotification>> {
        let subscribe_method = subscribe_method.into();
        let unsubscribe_method = unsubscribe_method.into();
        log::debug!(
            "[frontend] Subscribe: method={}/{}, params={:?}",
            subscribe_method,
            unsubscribe_method,
            params
        );
        let (tx, rx) = oneshot::channel();
        self.to_back
            .clone()
            .send(ToBackTaskMessage::Subscribe {
                subscribe_method,
                unsubscribe_method,
                params,
                send_back: tx,
            })
            .await
            .map_err(|_| ClientError::InternalChannel)?;

        let res = if let Some(duration) = self.timeout {
            #[cfg(feature = "ws-async-std")]
            let timeout = async_std::task::sleep(duration);
            #[cfg(feature = "ws-tokio")]
            let timeout = tokio::time::sleep(duration);
            futures::pin_mut!(rx, timeout);
            match future::select(rx, timeout).await {
                future::Either::Left((response, _)) => response,
                future::Either::Right((_, _)) => return Err(ClientError::WsRequestTimeout),
            }
        } else {
            rx.await
        };
        match res {
            Ok(Ok((id, notification_rx))) => Ok(WsSubscription {
                id,
                notification_rx,
                to_back: self.to_back.clone(),
            }),
            Ok(Err(err)) => Err(err),
            Err(_) => Err(ClientError::InternalChannel),
        }
    }

    /// Sends a unsubscribe request to the server.
    async fn send_unsubscribe(&self, unsubscribe_method: impl Into<String>, subscription_id: Id) -> Result<Output> {
        let subscription_id = serde_json::to_value(subscription_id)?;
        let params = Params::Array(vec![subscription_id]);
        self.send_request(unsubscribe_method, Some(params)).await
    }
}

/// Active subscription on a websocket client.
pub struct WsSubscription<Notif> {
    /// Subscription ID.
    pub id: Id,
    /// Channel from which we receive notifications from the server.
    notification_rx: mpsc::Receiver<Notif>,
    /// Channel to send unsubscribe request to the background task.
    to_back: mpsc::Sender<ToBackTaskMessage>,
}

impl<Notif> WsSubscription<Notif> {
    /// Returns the next notification from the websocket stream.
    ///
    /// Ignore any malformed packet.
    pub async fn next(&mut self) -> Option<Notif> {
        self.notification_rx.next().await
    }
}

impl<Notif> Stream for WsSubscription<Notif> {
    type Item = Notif;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        mpsc::Receiver::<Notif>::poll_next(Pin::new(&mut self.notification_rx), cx)
    }
}

impl<Notif> Drop for WsSubscription<Notif> {
    fn drop(&mut self) {
        let id = std::mem::replace(&mut self.id, Id::Num(0));
        let _ = self
            .to_back
            .send(ToBackTaskMessage::SubscriptionClosed(id))
            .now_or_never();
    }
}

#[async_trait::async_trait]
impl Transport for WsClient {
    async fn request<M>(&self, method: M, params: Option<Params>) -> Result<Output>
    where
        M: Into<String> + Send,
    {
        self.send_request(method, params).await
    }
}

#[async_trait::async_trait]
impl BatchTransport for WsClient {
    async fn request_batch<I, M>(&self, batch: I) -> Result<Vec<Output>>
    where
        I: IntoIterator<Item = (M, Option<Params>)> + Send,
        I::IntoIter: Send,
        M: Into<String>,
    {
        self.send_request_batch(batch).await
    }
}

#[async_trait::async_trait]
impl PubsubTransport for WsClient {
    type NotificationStream = WsSubscription<SubscriptionNotification>;

    async fn subscribe<M>(
        &self,
        subscribe_method: M,
        unsubscribe_method: M,
        params: Option<Params>,
    ) -> Result<(Id, Self::NotificationStream)>
    where
        M: Into<String> + Send,
    {
        let notification_stream = self
            .send_subscribe(subscribe_method, unsubscribe_method, params)
            .await?;
        Ok((notification_stream.id.clone(), notification_stream))
    }

    async fn unsubscribe<M>(&self, unsubscribe_method: M, subscription_id: Id) -> Result<bool>
    where
        M: Into<String> + Send,
    {
        let output = self.send_unsubscribe(unsubscribe_method, subscription_id).await?;
        match output {
            Output::Success(Success { result, .. }) => Ok(serde_json::from_value::<bool>(result)?),
            Output::Failure(failure) => {
                log::warn!("Unexpected unsubscribe response: {}", failure);
                Ok(false)
            }
        }
    }
}
