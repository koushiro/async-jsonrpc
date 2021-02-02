mod builder;
mod manager;
mod task;
#[cfg(test)]
mod tests;

use std::marker::PhantomData;

use futures::{
    channel::{mpsc, oneshot},
    future::FutureExt,
    sink::SinkExt,
    stream::StreamExt,
};
use jsonrpc_types::*;
use serde::de::DeserializeOwned;

pub use self::builder::WsTransportBuilder;
use crate::error::{Result, RpcClientError};

/// Message that the client can send to the background task.
pub(crate) enum ToBackTaskMessage {
    Notification {
        method: String,
        params: Option<Params>,
    },
    Request {
        method: String,
        params: Option<Params>,
        send_back: oneshot::Sender<Result<Value>>,
    },
    Subscribe {
        subscribe_method: String,
        unsubscribe_method: String,
        params: Option<Params>,
        send_back: oneshot::Sender<Result<(Id, mpsc::Receiver<Value>)>>,
    },
    SubscriptionClosed(Id),
}

/// WebSocket transport
pub struct WsTransport {
    to_back: mpsc::Sender<ToBackTaskMessage>,
}

impl WsTransport {
    /// Creates a new WebSocket transport.
    pub async fn new(url: impl Into<String>) -> Result<Self> {
        WsTransportBuilder::new().build(url).await
    }

    /// Creates a `WsTransportBuilder` to configure a `WsTransport`.
    ///
    /// This is the same as `WsTransportBuilder::new()`.
    pub fn builder() -> WsTransportBuilder {
        WsTransportBuilder::new()
    }

    /// Sends a `notification` request to the server.
    pub async fn notification(
        &self,
        method: impl Into<String>,
        params: Option<Params>,
    ) -> Result<()> {
        let method = method.into();
        log::trace!(
            "[frontend] send request: method={}, params={:?}",
            method,
            params
        );
        self.to_back
            .clone()
            .send(ToBackTaskMessage::Notification { method, params })
            .await
            .map_err(|_| RpcClientError::InternalChannel)
    }

    /// Sends a `method call` request to the server.
    pub async fn request<T>(&self, method: impl Into<String>, params: Option<Params>) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let method = method.into();
        log::trace!(
            "[frontend] send request: method={}, params={:?}",
            method,
            params
        );
        let (tx, rx) = oneshot::channel();
        self.to_back
            .clone()
            .send(ToBackTaskMessage::Request {
                method,
                params,
                send_back: tx,
            })
            .await
            .map_err(|_| RpcClientError::InternalChannel)?;

        let value = match rx.await {
            Ok(Ok(value)) => value,
            Ok(Err(err)) => return Err(err),
            Err(_) => return Err(RpcClientError::InternalTaskFinish),
        };
        Ok(serde_json::from_value(value)?)
    }

    /// Sends a subscribe request to the server.
    ///
    /// `subscribe_method` and `params` are used to ask for the subscription towards the server.
    /// `unsubscribe_method` is used to close the subscription.
    pub async fn subscribe<Notif>(
        &self,
        subscribe_method: impl Into<String>,
        unsubscribe_method: impl Into<String>,
        params: Option<Params>,
    ) -> Result<WsSubscription<Notif>> {
        let subscribe_method = subscribe_method.into();
        let unsubscribe_method = unsubscribe_method.into();
        log::trace!(
            "[frontend] subscribe: method={}/{}, params={:?}",
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
            .map_err(|_| RpcClientError::InternalChannel)?;

        let (id, notification_rx) = match rx.await {
            Ok(Ok(value)) => value,
            Ok(Err(err)) => return Err(err),
            Err(_) => return Err(RpcClientError::InternalTaskFinish),
        };
        Ok(WsSubscription {
            marker: PhantomData,
            to_back: self.to_back.clone(),
            notification_rx,
            id,
        })
    }
}

/// Active subscription on a websocket client.
pub struct WsSubscription<Notif> {
    marker: PhantomData<Notif>,
    // Subscription ID.
    id: Id,
    /// Channel from which we receive notifications from the server.
    notification_rx: mpsc::Receiver<Value>,
    /// Channel to send unsubscribe request to the background task.
    to_back: mpsc::Sender<ToBackTaskMessage>,
}

impl<Notif> WsSubscription<Notif>
where
    Notif: DeserializeOwned,
{
    /// Returns the next notification from the websocket stream.
    ///
    /// Ignore any malformed packet.
    pub async fn next(&mut self) -> Option<Notif> {
        loop {
            match self.notification_rx.next().await {
                Some(value) => match serde_json::from_value(value) {
                    Ok(res) => return Some(res),
                    Err(err) => log::error!("Subscription response error: {}", err),
                },
                None => return None,
            }
        }
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
