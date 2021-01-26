use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use async_tungstenite::{
    tokio::connect_async,
    tungstenite::{
        handshake::client::Request as HandShakeRequest, http::header, protocol::Message,
    },
};
use futures::{
    channel::{mpsc, oneshot},
    future,
    stream::StreamExt,
};
use jsonrpc_types::*;
use parking_lot::Mutex;
use serde::de::DeserializeOwned;

use crate::{error::Result, transports::Transport};

type Pending = oneshot::Sender<Result<Response>>;
type Pendings = Arc<Mutex<BTreeMap<Id, Pending>>>;
type Subscription = mpsc::UnboundedSender<Notification>;
type Subscriptions = Arc<Mutex<BTreeMap<Id, Subscription>>>;

type WsMsgSender = mpsc::UnboundedSender<Message>;
type WsMsgReceiver = mpsc::UnboundedReceiver<Message>;

enum TransportMessage {
    Request {
        id: Id,
        request: String,
        sender: Pending,
    },
    Subscribe {
        id: Id,
        sink: Subscription,
    },
    Unsubscribe {
        id: Id,
    },
}

///
pub struct WsTransport {
    id: Arc<AtomicU64>,
    url: String,
    pendings: Pendings,
    subscriptions: Subscriptions,
    sender: WebSocketSender,
    _handle: tokio::task::JoinHandle<()>,
}

impl WsTransport {
    /// Create a new WebSocket transport with given `url`.
    pub fn new<U: Into<String>>(url: U) -> Self {
        let url = url.into();
        let handshake_request = HandShakeRequest::get(&url)
            .body(())
            .expect("Handshake HTTP request should be valid");

        let pending = Arc::new(Mutex::new(BTreeMap::new()));
        let subscriptions = Arc::new(Mutex::new(BTreeMap::new()));
        let (writer_tx, writer_rx) = mpsc::unbounded();

        let handle = tokio::task::spawn(ws_task(
            handshake_request,
            pending.clone(),
            subscriptions.clone(),
            writer_tx.clone(),
            writer_rx,
        ));

        Self {
            id: Arc::new(AtomicU64::new(1)),
            url,
            pendings: pending,
            subscriptions,
            sender: writer_tx,
            _handle: handle,
        }
    }

    /// Create a new WebSocket transport with given `url` and bearer `token`.
    pub fn new_with_bearer_auth<U: Into<String>, T: Into<String>>(url: U, token: T) -> Self {
        let url = url.into();
        let token = token.into();

        let bearer_auth_header_value = format!("Bearer {}", token);
        let handshake_request = HandShakeRequest::get(&url)
            .header(header::AUTHORIZATION, bearer_auth_header_value)
            .body(())
            .expect("Handshake HTTP request should be valid");

        let pending = Arc::new(Mutex::new(BTreeMap::new()));
        let subscriptions = Arc::new(Mutex::new(BTreeMap::new()));
        let (writer_tx, writer_rx) = mpsc::unbounded();

        let handle = tokio::task::spawn(ws_task(
            handshake_request,
            pending.clone(),
            subscriptions.clone(),
            writer_tx.clone(),
            writer_rx,
        ));

        Self {
            id: Arc::new(AtomicU64::new(1)),
            url,
            pendings: pending,
            subscriptions,
            sender: writer_tx,
            _handle: handle,
        }
    }

    async fn send_request(&self, request: Request) -> Result<Response> {
        let request = serde_json::to_string(&request)?;
        log::debug!("Calling: {}", request);
        let (tx, rx) = oneshot::channel();
        self.pendings.lock().insert(id, tx);
        self.sender
            .unbounded_send(Message::Text(request))
            .expect("Sending `Text` Message should be successful");

        rx.await.unwrap()
    }
}

async fn ws_task(
    handshake_request: HandShakeRequest,
    pendings: Pendings,
    sub: Subscriptions,
    tx: WebSocketSender,
    rx: WebSocketReceiver,
) {
    let (ws_stream, _) = connect_async(handshake_request)
        .await
        .expect("Handshake request is valid, but failed to connect");
    log::info!("WebSocket handshake has been successfully completed");
    let (sink, stream) = ws_stream.split();

    // receive request from WebSocketSender,
    // and forward the request to sink that will send message to websocket stream.
    let write_to_ws = rx.map(Ok).forward(sink);
    // read websocket message from websocket stream, and handle the incoming message.
    let read_from_ws = stream.for_each(|msg| async {
        match msg {
            Ok(msg) => handle_incoming_msg(msg, pendings.clone(), sub.clone(), tx.clone()),
            Err(err) => log::error!("WebSocket stream read error: {}", err),
        }
    });

    futures::pin_mut!(write_to_ws, read_from_ws);
    future::select(write_to_ws, read_from_ws).await;
}

fn handle_incoming_msg(
    msg: Message,
    pendings: Pendings,
    subscriptions: Subscriptions,
    tx: WebSocketSender,
) {
    match msg {
        Message::Text(msg) => {
            handle_subscription(subscriptions, &msg);
            handle_pending_response(pendings, &msg);
        }
        Message::Binary(msg) => log::warn!("Receive `Binary` Message: {:?}", msg),
        Message::Close(msg) => {
            log::warn!("Receive `Close` Message: {:?}", msg);
            tx.unbounded_send(Message::Close(msg))
                .expect("Sending `Close` Message should be successful")
        }
        Message::Ping(msg) => {
            log::warn!("Receive `Ping` Message: {:?}", msg);
            tx.unbounded_send(Message::Pong(msg))
                .expect("Sending `Pong` Message should be successful")
        }
        Message::Pong(msg) => log::warn!("Receive `Pong` Message: {:?}", msg),
    }
}

fn handle_subscription(subscriptions: Subscriptions, msg: &str) {
    if let Ok(notification) = serde_json::from_str::<Notification>(msg) {
        if let Params::Array(params) = notification.params {
            let id = params.get(0);
            let result = params.get(1);
            if let (Some(Value::Number(id)), Some(result)) = (id, result) {
                let id = id.as_u64().unwrap() as usize;
                if let Some(stream) = subscriptions.lock().get(&id) {
                    stream
                        .unbounded_send(result.clone())
                        .expect("Sending subscription result to the user should be successful");
                } else {
                    log::warn!("Got notification for unknown subscription (id: {})", id);
                }
            } else {
                log::error!("Got unsupported notification (id: {:?})", id);
            }
        } else {
            log::error!(
                "The Notification Params is not JSON array type: {}",
                serde_json::to_string(&notification.params)
                    .expect("Serialize `Params` never fails")
            );
        }
    }
}

fn handle_pending_response(pendings: Pendings, msg: &str) {
    let response = serde_json::from_str::<Response>(msg).map_err(Into::into);
    let id = match &response {
        Ok(Response::Single(output)) => output.id(),
        Ok(Response::Batch(outputs)) => outputs.get(0).map_or(0, |output| output.id()),
        Err(_) => 0,
    };
    if let Some(request) = pendings.lock().remove(&id) {
        if let Err(err) = request.send(response) {
            log::error!("Sending a response to deallocated channel: {:?}", err);
        }
    }
}

#[async_trait::async_trait]
impl Transport for WsTransport {
    fn prepare<M: Into<String>>(&self, method: M, params: Option<Params>) -> Call {
        let id = self.id.fetch_add(1, Ordering::AcqRel);
        Call::MethodCall(MethodCall {
            jsonrpc: Some(Version::V2_0),
            method: method.into(),
            params,
            id: Id::Num(id),
        })
    }

    async fn execute(&self, request: Request) -> Result<Response> {
        self.send_request(request).await
    }
}

/*
impl PubsubTransport for WsTransport {
    fn subscribe<T>(&self, id: SubscriptionId) -> NotificationStream<T>
    where
        T: DeserializeOwned,
    {
        let (tx, rx) = mpsc::unbounded();
        if self.subscriptions.lock().insert(id, tx).is_some() {
            log::warn!("Replacing already-registered subscription with id {:?}", id);
        }
        Box::pin(
            rx.map(|value| serde_json::from_value(value).expect("Deserialize `Value` never fails")),
        )
    }

    fn unsubscribe(&self, id: SubscriptionId) {
        self.subscriptions.lock().remove(&id);
    }
}
*/
