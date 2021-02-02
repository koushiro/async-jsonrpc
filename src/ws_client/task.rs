#[cfg(feature = "ws-async-std")]
use async_tungstenite::async_std::{connect_async, ConnectStream};
#[cfg(feature = "ws-tokio")]
use async_tungstenite::tokio::{connect_async, ConnectStream};
use async_tungstenite::{
    tungstenite::{
        error::Error as WsError, handshake::client::Request as HandShakeRequest, protocol::Message,
    },
    WebSocketStream,
};
use futures::{
    channel::mpsc,
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};
use jsonrpc_types::*;

use crate::ws_client::{manager::TaskManager, ToBackTaskMessage};

type WsMsgSender = SplitSink<WebSocketStream<ConnectStream>, Message>;
type WsMsgReceiver = SplitStream<WebSocketStream<ConnectStream>>;

struct WsSender {
    id: u64,
    sender: WsMsgSender,
}

impl WsSender {
    fn new(sender: WsMsgSender) -> Self {
        Self { id: 0, sender }
    }

    async fn send_message(&mut self, msg: Message) -> Result<(), WsError> {
        log::trace!("[background] send websocket message: {}", msg);
        self.sender.feed(msg).await?;
        self.sender.flush().await?;
        Ok(())
    }

    async fn send_notification(
        &mut self,
        method: impl Into<String>,
        params: Option<Params>,
    ) -> Result<(), WsError> {
        let request = Request::Single(Call::Notification(Notification::new(method, params)));
        let request = serde_json::to_string(&request).expect("`Notification` shouldn't be failed");
        self.send_message(Message::Text(request)).await
    }

    async fn send_request(
        &mut self,
        method: impl Into<String>,
        params: Option<Params>,
    ) -> Result<u64, WsError> {
        let id = self.id;
        self.id = id.wrapping_add(1);

        let request = Request::Single(Call::MethodCall(MethodCall::new(
            method,
            params,
            Id::Num(id),
        )));
        let request = serde_json::to_string(&request).expect("`Request` shouldn't be failed");
        self.send_message(Message::Text(request)).await?;
        Ok(id)
    }

    async fn start_subscription(
        &mut self,
        method: impl Into<String>,
        params: Option<Params>,
    ) -> Result<u64, WsError> {
        self.send_request(method, params).await
    }
}

struct WsReceiver(WsMsgReceiver);
impl WsReceiver {
    fn new(receiver: WsMsgReceiver) -> Self {
        Self(receiver)
    }

    async fn recv_message(&mut self) -> Result<Message, WsError> {
        loop {
            if let Some(message) = self.0.next().await {
                let message = message?;
                log::trace!("[background] recv websocket message: {}", message);
                return Ok(message);
            }
        }
    }
}

pub(crate) struct WsTask {
    sender: WsSender,
    receiver: WsReceiver,
    manager: TaskManager,
}

impl WsTask {
    pub(crate) async fn handshake(request: HandShakeRequest) -> Result<Self, WsError> {
        let uri = request.uri().clone();
        log::debug!("WebSocket handshake {}, request: {:?}", uri, request);
        let (ws_stream, response) = connect_async(request).await?;
        log::debug!("WebSocket handshake {}, response: {:?}", uri, response);
        let (sink, stream) = ws_stream.split();
        Ok(Self {
            sender: WsSender::new(sink),
            receiver: WsReceiver::new(stream),
            manager: TaskManager::new(),
        })
    }

    pub(crate) async fn into_task(self, mut from_front: mpsc::Receiver<ToBackTaskMessage>) {
        let Self {
            mut sender,
            receiver,
            mut manager,
        } = self;

        let from_back = futures::stream::unfold(receiver, |mut receiver| async {
            let res = receiver.recv_message().await;
            Some((res, receiver))
        });
        futures::pin_mut!(from_front, from_back);

        loop {
            futures::select! {
                msg = from_front.next() => match msg {
                    Some(ToBackTaskMessage::Notification { method, params }) => {
                        log::trace!("[backend] prepare to send notification");
                        let _ = sender.send_notification(method, params).await;
                    }
                    Some(ToBackTaskMessage::Request { method, params, send_back }) => {

                    }
                    Some(ToBackTaskMessage::Subscribe { subscribe_method, unsubscribe_method, params, send_back }) => {

                    }
                    Some(ToBackTaskMessage::SubscriptionClosed(id)) => {

                    }
                    None => {
                        log::error!("[backend] frontend channel dropped; terminate client");
                        break;
                    }
                },
                msg = from_back.next() => match msg {
                    Some(_) => {}
                    None => {}
                },
            }
        }
    }
}
