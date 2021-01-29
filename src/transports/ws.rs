use std::{
    collections::BTreeMap,
    fmt,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

#[cfg(feature = "ws-async-std")]
use async_tungstenite::async_std::{connect_async, ConnectStream};
#[cfg(feature = "ws-tokio")]
use async_tungstenite::tokio::{connect_async, ConnectStream};
use async_tungstenite::{
    tungstenite::{
        handshake::client::Request as HandShakeRequest,
        http::header::{self, HeaderMap, HeaderName, HeaderValue},
        protocol::Message,
    },
    WebSocketStream,
};
use futures::{
    channel::{mpsc, oneshot},
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};
use jsonrpc_types::*;

use crate::{
    error::{Result, RpcClientError},
    transports::{BatchTransport, PubsubTransport, Transport},
};

/// A `WsTransportBuilder` can be used to create a `HttpTransport` with  custom configuration.
#[derive(Debug)]
pub struct WsTransportBuilder {
    headers: HeaderMap,
}

impl Default for WsTransportBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl WsTransportBuilder {
    /// Creates a new `WsTransportBuilder`.
    ///
    /// This is the same as `WsTransport::builder()`.
    pub fn new() -> Self {
        Self {
            headers: HeaderMap::new(),
        }
    }

    // ========================================================================
    // HTTP header options
    // ========================================================================

    /// Enable basic authentication.
    pub fn basic_auth<U, P>(self, username: U, password: Option<P>) -> Self
    where
        U: fmt::Display,
        P: fmt::Display,
    {
        let mut basic_auth = "Basic ".to_string();
        let auth = if let Some(password) = password {
            base64::encode(format!("{}:{}", username, password))
        } else {
            base64::encode(format!("{}:", username))
        };
        basic_auth.push_str(&auth);
        let value = HeaderValue::from_str(&basic_auth).expect("basic auth header value");
        self.header(header::AUTHORIZATION, value)
    }

    /// Enable bearer authentication.
    pub fn bearer_auth<T>(self, token: T) -> Self
    where
        T: fmt::Display,
    {
        let bearer_auth = format!("Bearer {}", token);
        let value = HeaderValue::from_str(&bearer_auth).expect("bearer auth header value");
        self.header(header::AUTHORIZATION, value)
    }

    /// Adds a `Header` for handshake request.
    pub fn header(mut self, name: HeaderName, value: HeaderValue) -> Self {
        self.headers.insert(name, value);
        self
    }

    /// Adds `Header`s for handshake request.
    pub fn headers(mut self, headers: HeaderMap) -> Self {
        self.headers.extend(headers);
        self
    }

    // ========================================================================

    /// Returns a `WsTransport` that uses this `WsTransportBuilder` configuration.
    pub async fn handshake<U: Into<String>>(self, url: U) -> Result<WsTransport> {
        let url = url.into();

        let mut handshake_builder = HandShakeRequest::get(&url);
        let headers = handshake_builder
            .headers_mut()
            .expect("HandShakeRequest just created");
        headers.extend(self.headers);
        let handshake_req = handshake_builder.body(())?;

        let task = WsTask::handshake(handshake_req).await?;

        let (msg_tx, msg_rx) = mpsc::unbounded();
        #[cfg(feature = "ws-async-std")]
        let _handle = async_std::task::spawn(task.into_task(msg_rx));
        #[cfg(feature = "ws-tokio")]
        let _handle = tokio::spawn(task.into_task(msg_rx));

        Ok(WsTransport {
            url,
            id: Arc::new(AtomicU64::new(1)),
            msg_tx,
        })
    }
}

type Pending = oneshot::Sender<Result<Response>>;
type Subscription = mpsc::UnboundedSender<SubscriptionNotification>;

struct WsTask {
    pendings: BTreeMap<Id, Pending>,
    subscriptions: BTreeMap<Id, Subscription>,
    sink: SplitSink<WebSocketStream<ConnectStream>, Message>,
    stream: SplitStream<WebSocketStream<ConnectStream>>,
}

impl WsTask {
    async fn handshake(request: HandShakeRequest) -> Result<Self> {
        let uri = request.uri().clone();
        log::debug!("WebSocket handshake {}, request: {:?}", uri, request);
        let (ws_stream, response) = connect_async(request).await?;
        log::debug!(
            "WebSocket handshake {} successfully, response: {:?}",
            uri,
            response
        );
        let (sink, stream) = ws_stream.split();
        Ok(Self {
            pendings: BTreeMap::new(),
            subscriptions: BTreeMap::new(),
            sink,
            stream,
        })
    }

    async fn into_task(self, msg_rx: WsMsgReceiver) {
        let Self {
            mut pendings,
            mut subscriptions,
            mut sink,
            stream,
        } = self;

        let msg_rx = msg_rx.fuse();
        let stream = stream.fuse();
        futures::pin_mut!(msg_rx, stream);

        loop {
            futures::select! {
                send_msg = msg_rx.next() => match send_msg {
                    Some(TransportMessage::Request { id, request, sender }) => {
                        if pendings.insert(id.clone(), sender).is_some() {
                            log::warn!("Replacing a pending request with id {:?}", id);
                        }
                        let request = serde_json::to_string(&request)
                            .expect("Serialize `MethodCallRequest` shouldn't be failed");
                        if let Err(err) = sink.send(Message::Text(request)).await {
                            log::error!("WebSocket connection error: {}", err);
                            pendings.remove(&id);
                        }
                    }
                    Some(TransportMessage::Subscribe { id, sender }) => {
                        if subscriptions.insert(id.clone(), sender).is_some() {
                            log::warn!("Replacing already-registered subscription with id {:?}", id);
                        }
                    }
                    Some(TransportMessage::Unsubscribe { id }) => {
                        if subscriptions.remove(&id).is_none() {
                            log::warn!("Unsubscribing from non-existent subscription with id {:?}", id);
                        }
                    }
                    None => {}
                },
                recv_msg = stream.next() => match recv_msg {
                    Some(Ok(msg)) => handle_message(msg, &mut pendings, &subscriptions, &mut sink).await,
                    Some(Err(err)) => {
                        log::error!("WebSocket connection error: {}", err);
                        break;
                    }
                    None => break,
                },
                complete => break,
            }
        }
    }
}

async fn handle_message(
    msg: Message,
    pendings: &mut BTreeMap<Id, Pending>,
    subscriptions: &BTreeMap<Id, Subscription>,
    sink: &mut SplitSink<WebSocketStream<ConnectStream>, Message>,
) {
    log::trace!("Message received: {:?}", msg);
    match msg {
        Message::Text(msg) => {
            handle_subscription(subscriptions, &msg);
            handle_pending_response(pendings, &msg);
        }
        Message::Binary(msg) => log::warn!("Receive `Binary` Message: {:?}", msg),
        Message::Close(msg) => {
            log::warn!("Receive `Close` Message: {:?}", msg);
            sink.send(Message::Close(msg))
                .await
                .expect("Sending `Close` Message should be successful")
        }
        Message::Ping(msg) => {
            log::warn!("Receive `Ping` Message: {:?}", msg);
            sink.send(Message::Pong(msg))
                .await
                .expect("Sending `Pong` Message should be successful")
        }
        Message::Pong(msg) => log::warn!("Receive `Pong` Message: {:?}", msg),
    }
}

fn handle_subscription(subscriptions: &BTreeMap<Id, Subscription>, msg: &str) {
    if let Ok(notification) = serde_json::from_str::<SubscriptionNotification>(msg) {
        let id = notification.params.subscription.clone();
        if let Some(stream) = subscriptions.get(&id) {
            stream
                .unbounded_send(notification)
                .expect("Sending subscription result to the user should be successful");
        } else {
            log::warn!("Got notification for unknown subscription (id: {:?})", id);
        }
    }
}

fn handle_pending_response(pendings: &mut BTreeMap<Id, Pending>, msg: &str) {
    let response = serde_json::from_str::<Response>(msg).map_err(Into::into);
    let id = match response {
        Ok(Response::Single(Output::Success(ref success))) => success.id.clone(),
        Ok(Response::Single(Output::Failure(ref failure))) => {
            failure.id.clone().unwrap_or_else(|| Id::Num(0))
        }
        Ok(Response::Batch(ref outputs)) => outputs
            .first()
            .map(|output| match output {
                Output::Success(success) => success.id.clone(),
                Output::Failure(failure) => failure.id.clone().unwrap_or_else(|| Id::Num(0)),
            })
            .unwrap_or_else(|| Id::Num(0)),
        Err(_) => Id::Num(0),
    };
    if let Some(request) = pendings.remove(&id) {
        if let Err(err) = request.send(response) {
            log::error!("Sending a response to deallocated channel: {:?}", err);
        }
    }
}

enum TransportMessage {
    Request {
        // if request is a batch of calls, use the minimum id.
        id: Id,
        request: MethodCallRequest,
        sender: Pending,
    },
    Subscribe {
        id: Id,
        sender: Subscription,
    },
    Unsubscribe {
        id: Id,
    },
}

type WsMsgSender = mpsc::UnboundedSender<TransportMessage>;
type WsMsgReceiver = mpsc::UnboundedReceiver<TransportMessage>;

/// WebSocket transport
pub struct WsTransport {
    url: String,
    id: Arc<AtomicU64>,
    msg_tx: WsMsgSender,
}

impl WsTransport {
    /// Creates a new WebSocket transport.
    pub async fn new<U: Into<String>>(url: U) -> Result<Self> {
        WsTransportBuilder::new().handshake(url).await
    }

    /// Creates a `WsTransportBuilder` to configure a `WsTransport`.
    ///
    /// This is the same as `WsTransportBuilder::new()`.
    pub fn builder() -> WsTransportBuilder {
        WsTransportBuilder::new()
    }

    /// Returns the websocket url.
    pub fn url(&self) -> &str {
        &self.url
    }

    // pub fn handle(&self) -> &

    fn send_msg(&self, msg: TransportMessage) -> Result<()> {
        self.msg_tx
            .unbounded_send(msg)
            .map_err(|_| RpcClientError::InternalTaskFinish)
    }

    async fn send_request(&self, request: MethodCallRequest) -> Result<Response> {
        let (sender, receiver) = oneshot::channel();
        let id = match &request {
            MethodCallRequest::Single(call) => call.id.clone(),
            MethodCallRequest::Batch(calls) => calls
                .iter()
                .map(|call| call.id.clone())
                .min()
                .expect("Batch of calls shouldn't be empty"),
        };
        self.send_msg(TransportMessage::Request {
            id,
            request,
            sender,
        })?;
        receiver
            .await
            .expect("Oneshot channel shouldn't be canceled")
    }
}

/*
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
*/

#[async_trait::async_trait]
impl Transport for WsTransport {
    fn prepare<M: Into<String>>(&self, method: M, params: Option<Params>) -> MethodCall {
        let id = self.id.fetch_add(1, Ordering::AcqRel);
        MethodCall {
            jsonrpc: Version::V2_0,
            method: method.into(),
            params,
            id: Id::Num(id),
        }
    }

    async fn execute(&self, call: MethodCall) -> Result<Response> {
        let request = MethodCallRequest::Single(call);
        self.send_request(request).await
    }
}

#[async_trait::async_trait]
impl BatchTransport for WsTransport {
    async fn execute_batch<I>(&self, calls: I) -> Result<Response, RpcClientError>
    where
        I: IntoIterator<Item = MethodCall> + Send,
        I::IntoIter: Send,
    {
        let request = MethodCallRequest::Batch(calls.into_iter().collect());
        self.send_request(request).await
    }
}

///
pub type NotificationStream = mpsc::UnboundedReceiver<SubscriptionNotification>;

impl PubsubTransport for WsTransport {
    type NotificationStream = NotificationStream;

    fn subscribe(&self, id: Id) -> Result<Self::NotificationStream> {
        let (sink, stream) = mpsc::unbounded();
        self.send_msg(TransportMessage::Subscribe { id, sender: sink })?;
        Ok(stream)
    }

    fn unsubscribe(&self, id: Id) -> Result<()> {
        self.send_msg(TransportMessage::Unsubscribe { id })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn websocket() {
        env_logger::init();

        let ws = WsTransport::new("wss://rpc.polkadot.io").await.unwrap();
        let response = ws.send("state_getRuntimeVersion", None).await.unwrap();
        log::info!("Response: {}", response);

        let response = ws
            .send_batch(vec![
                ("state_getRuntimeVersion", None),
                ("system_name", None),
            ])
            .await
            .unwrap();
        log::info!("Response: {}", response);

        let response = ws.send("chain_subscribeNewHead", None).await.unwrap();
        let id = match response {
            Response::Single(Output::Success(Success { result, .. })) => {
                serde_json::from_value::<Id>(result).unwrap()
            }
            _ => panic!("Unknown"),
        };
        let mut stream = ws.subscribe(id).unwrap();
        while let Some(value) = stream.next().await {
            log::info!(
                "chain_subscribeNewHead: {}",
                serde_json::to_string(&value).unwrap()
            );
        }
    }
}
