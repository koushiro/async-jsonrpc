use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use parking_lot::Mutex;
use soketto::{Sender, Receiver};
use tokio::net::TcpStream;

use crate::errors::Result;
use crate::transports::{BatchTransport, DuplexTransport, Transport};
use crate::types::{Call, MethodCall, Params, Request, Response, SubscriptionId, Version};

#[derive(Clone)]
pub struct WebSocketTransport {
    id: Arc<AtomicUsize>,
    url: String,
    sender: Sender<TcpStream>,
    receiver: Receiver<TcpStream>,
}

impl WebSocketTransport {
    pub async fn connect<U: Into<String>>(url: U) -> Result<Self> {
        let url = url.into();


        Ok(Self {
            id: Default::default(),
            url,
            sender
        })
    }
}

#[async_trait::async_trait(?Send)]
impl Transport for WebSocketTransport {
    fn prepare<M: Into<String>>(&self, method: M, params: Params) -> Call {
        let id = self.id.fetch_add(1, Ordering::AcqRel);
        Call::MethodCall(MethodCall {
            jsonrpc: Some(Version::V2),
            id,
            method: method.into(),
            params,
        })
    }

    async fn execute(&self, request: &Request) -> Result<Response> {
        let request = serde_json::to_string(request)?;
        self.writer.send(Message::Text(request)).await?;
    }
}

#[async_trait::async_trait(?Send)]
impl BatchTransport for WebSocketTransport {}

#[async_trait::async_trait(?Send)]
impl DuplexTransport for WebSocketTransport {
    type NotificationStream = ();

    async fn subscribe(&self, id: &SubscriptionId) -> Self::NotificationStream {
        unimplemented!()
    }

    fn unsubscribe(&self, id: &SubscriptionId) {
        unimplemented!()
    }
}
