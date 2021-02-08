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
    future,
    sink::SinkExt,
    stream::{Stream, StreamExt},
};
use jsonrpc_types::*;

pub use self::builder::WsClientBuilder;
use crate::{
    error::WsClientError,
    transport::{BatchTransport, PubsubTransport, Transport},
};

/// Message that the client can send to the background task.
pub(crate) enum ToBackTaskMessage {
    Request {
        method: String,
        params: Option<Params>,
        /// One-shot channel where to send back the response of the request.
        send_back: oneshot::Sender<Result<Output, WsClientError>>,
    },
    BatchRequest {
        batch: Vec<(String, Option<Params>)>,
        /// One-shot channel where to send back the response of the batch request.
        send_back: oneshot::Sender<Result<Vec<Output>, WsClientError>>,
    },
    Subscribe {
        subscribe_method: String,
        params: Option<Params>,
        /// One-shot channel where to send back the response (subscription id) and a `Receiver`
        /// that will receive subscription notification when we get a response (subscription id)
        /// from the server about the subscription.
        send_back: oneshot::Sender<Result<(Id, mpsc::Receiver<SubscriptionNotification>), WsClientError>>,
    },
    Unsubscribe {
        unsubscribe_method: String,
        subscription_id: Id,
        /// One-shot channel where to send back the response of the unsubscribe request.
        send_back: oneshot::Sender<Result<bool, WsClientError>>,
    },
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
    pub async fn new(url: impl Into<String>) -> Result<Self, WsClientError> {
        WsClientBuilder::new()
            .build(url)
            .await
            .map_err(WsClientError::WebSocket)
    }

    /// Creates a `WsClientBuilder` to configure a `WsClient`.
    ///
    /// This is the same as `WsClientBuilder::new()`.
    pub fn builder() -> WsClientBuilder {
        WsClientBuilder::new()
    }

    /// Sends a `method call` request to the server.
    async fn send_request(&self, method: impl Into<String>, params: Option<Params>) -> Result<Output, WsClientError> {
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
            .map_err(|_| WsClientError::InternalChannel)?;

        let res = if let Some(duration) = self.timeout {
            #[cfg(feature = "ws-async-std")]
            let timeout = async_std::task::sleep(duration);
            #[cfg(feature = "ws-tokio")]
            let timeout = tokio::time::sleep(duration);
            futures::pin_mut!(rx, timeout);
            match future::select(rx, timeout).await {
                future::Either::Left((response, _)) => response,
                future::Either::Right((_, _)) => return Err(WsClientError::RequestTimeout),
            }
        } else {
            rx.await
        };
        match res {
            Ok(Ok(output)) => Ok(output),
            Ok(Err(err)) => Err(err),
            Err(_) => Err(WsClientError::InternalChannel),
        }
    }

    /// Sends a batch of `method call` requests to the server.
    async fn send_request_batch<I, M>(&self, batch: I) -> Result<Vec<Output>, WsClientError>
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
            .map_err(|_| WsClientError::InternalChannel)?;

        let res = if let Some(duration) = self.timeout {
            #[cfg(feature = "ws-async-std")]
            let timeout = async_std::task::sleep(duration);
            #[cfg(feature = "ws-tokio")]
            let timeout = tokio::time::sleep(duration);
            futures::pin_mut!(rx, timeout);
            match future::select(rx, timeout).await {
                future::Either::Left((response, _)) => response,
                future::Either::Right((_, _)) => return Err(WsClientError::RequestTimeout),
            }
        } else {
            rx.await
        };
        match res {
            Ok(Ok(outputs)) => Ok(outputs),
            Ok(Err(err)) => Err(err),
            Err(_) => Err(WsClientError::InternalChannel),
        }
    }

    /// Sends a subscribe request to the server.
    ///
    /// `subscribe_method` and `params` are used to ask for the subscription towards the server.
    /// `unsubscribe_method` is used to close the subscription.
    async fn send_subscribe(
        &self,
        subscribe_method: impl Into<String>,
        params: Option<Params>,
    ) -> Result<WsSubscription<SubscriptionNotification>, WsClientError> {
        let subscribe_method = subscribe_method.into();
        log::debug!("[frontend] Subscribe: method={}, params={:?}", subscribe_method, params);
        let (tx, rx) = oneshot::channel();
        self.to_back
            .clone()
            .send(ToBackTaskMessage::Subscribe {
                subscribe_method,
                params,
                send_back: tx,
            })
            .await
            .map_err(|_| WsClientError::InternalChannel)?;

        let res = if let Some(duration) = self.timeout {
            #[cfg(feature = "ws-async-std")]
            let timeout = async_std::task::sleep(duration);
            #[cfg(feature = "ws-tokio")]
            let timeout = tokio::time::sleep(duration);
            futures::pin_mut!(rx, timeout);
            match future::select(rx, timeout).await {
                future::Either::Left((response, _)) => response,
                future::Either::Right((_, _)) => return Err(WsClientError::RequestTimeout),
            }
        } else {
            rx.await
        };
        match res {
            Ok(Ok((id, notification_rx))) => Ok(WsSubscription { id, notification_rx }),
            Ok(Err(err)) => Err(err),
            Err(_) => Err(WsClientError::InternalChannel),
        }
    }

    /// Sends an unsubscribe request to the server.
    async fn send_unsubscribe(
        &self,
        unsubscribe_method: impl Into<String>,
        subscription_id: Id,
    ) -> Result<bool, WsClientError> {
        let unsubscribe_method = unsubscribe_method.into();
        log::debug!(
            "[frontend] unsubscribe: method={}, id={:?}",
            unsubscribe_method,
            subscription_id
        );
        let (tx, rx) = oneshot::channel();
        self.to_back
            .clone()
            .send(ToBackTaskMessage::Unsubscribe {
                unsubscribe_method,
                subscription_id,
                send_back: tx,
            })
            .await
            .map_err(|_| WsClientError::InternalChannel)?;

        let res = if let Some(duration) = self.timeout {
            #[cfg(feature = "ws-async-std")]
            let timeout = async_std::task::sleep(duration);
            #[cfg(feature = "ws-tokio")]
            let timeout = tokio::time::sleep(duration);
            futures::pin_mut!(rx, timeout);
            match future::select(rx, timeout).await {
                future::Either::Left((response, _)) => response,
                future::Either::Right((_, _)) => return Err(WsClientError::RequestTimeout),
            }
        } else {
            rx.await
        };

        match res {
            Ok(Ok(res)) => Ok(res),
            Ok(Err(err)) => Err(err),
            Err(_) => Err(WsClientError::InternalChannel),
        }
    }
}

/// Active subscription on a websocket client.
pub struct WsSubscription<Notif> {
    /// Subscription ID.
    pub id: Id,
    /// Channel from which we receive notifications from the server.
    notification_rx: mpsc::Receiver<Notif>,
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

#[async_trait::async_trait]
impl Transport for WsClient {
    type Error = WsClientError;

    async fn request<M>(&self, method: M, params: Option<Params>) -> Result<Output, Self::Error>
    where
        M: Into<String> + Send,
    {
        self.send_request(method, params).await
    }
}

#[async_trait::async_trait]
impl BatchTransport for WsClient {
    async fn request_batch<I, M>(&self, batch: I) -> Result<Vec<Output>, <Self as Transport>::Error>
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
        params: Option<Params>,
    ) -> Result<(Id, Self::NotificationStream), <Self as Transport>::Error>
    where
        M: Into<String> + Send,
    {
        let notification_stream = self.send_subscribe(subscribe_method, params).await?;
        Ok((notification_stream.id.clone(), notification_stream))
    }

    async fn unsubscribe<M>(
        &self,
        unsubscribe_method: M,
        subscription_id: Id,
    ) -> Result<bool, <Self as Transport>::Error>
    where
        M: Into<String> + Send,
    {
        self.send_unsubscribe(unsubscribe_method, subscription_id).await
    }
}
