#[cfg(feature = "ws-async-std")]
use async_tungstenite::async_std::{connect_async, ConnectStream};
#[cfg(feature = "ws-tokio")]
use async_tungstenite::tokio::{connect_async, ConnectStream};
use async_tungstenite::{
    tungstenite::{handshake::client::Request as HandShakeRequest, protocol::Message},
    WebSocketStream,
};
use futures::{
    channel::mpsc,
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};
use jsonrpc_types::*;

use crate::{
    error::{WsClientError, WsError},
    ws_client::{
        manager::{RequestStatus, TaskManager},
        ToBackTaskMessage,
    },
};

type WsMsgSender = SplitSink<WebSocketStream<ConnectStream>, Message>;
type WsMsgReceiver = SplitStream<WebSocketStream<ConnectStream>>;

struct WsSender {
    id: u64,
    sender: WsMsgSender,
}

impl WsSender {
    fn new(sender: WsMsgSender) -> Self {
        Self { id: 1, sender }
    }

    async fn send_message(&mut self, msg: Message) -> Result<(), WsError> {
        log::debug!("[backend] Send websocket message: {}", msg);
        self.sender.feed(msg).await?;
        self.sender.flush().await?;
        Ok(())
    }

    async fn send_request(&mut self, method: impl Into<String>, params: Option<Params>) -> Result<u64, WsError> {
        let method = method.into();
        log::debug!("[backend] Send method call: method={}, params={:?}", method, params);

        let id = self.id;
        self.id = id.wrapping_add(1);
        let call = Call::MethodCall(MethodCall::new(method, params, Id::Num(id)));
        let request = serde_json::to_string(&call).expect("serialize call; qed");
        self.send_message(Message::Text(request)).await?;
        Ok(id)
    }

    async fn send_batch_request<I, M>(&mut self, batch: I) -> Result<Vec<u64>, WsError>
    where
        I: IntoIterator<Item = (M, Option<Params>)>,
        M: Into<String>,
    {
        let mut calls = vec![];
        let mut ids = vec![];
        for (method, params) in batch {
            let method = method.into();
            let id = self.id;
            self.id = id.wrapping_add(1);
            let call = Call::MethodCall(MethodCall::new(method, params, Id::Num(id)));
            ids.push(id);
            calls.push(call);
        }
        log::debug!("[backend] Send a batch of method calls: {:?}", calls);
        let request = Request::Batch(calls);
        let request = serde_json::to_string(&request).expect("serialize calls; qed");
        self.send_message(Message::Text(request)).await?;
        Ok(ids)
    }

    async fn start_subscription(
        &mut self,
        subscribe_method: impl Into<String>,
        params: Option<Params>,
    ) -> Result<u64, WsError> {
        self.send_request(subscribe_method, params).await
    }

    async fn stop_subscription(
        &mut self,
        unsubscribe_method: impl Into<String>,
        subscription_id: Id,
    ) -> Result<u64, WsError> {
        let subscription_id = serde_json::to_value(subscription_id).expect("serialize Id");
        let params = Params::Array(vec![subscription_id]);
        self.send_request(unsubscribe_method, Some(params)).await
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
                log::debug!("[backend] Receive websocket message: {}", message);
                return Ok(message);
            }
        }
    }
}

/// Helper struct for managing tasks on a websocket connection.
pub(crate) struct WsTask {
    sender: WsSender,
    receiver: WsReceiver,
    manager: TaskManager,
}

impl WsTask {
    /// Setup websocket connection.
    pub(crate) async fn handshake(
        request: HandShakeRequest,
        max_capacity_per_subscription: usize,
    ) -> Result<Self, WsError> {
        let uri = request.uri().clone();
        log::debug!("WebSocket handshake {}, request: {:?}", uri, request);
        let (ws_stream, response) = connect_async(request).await?;
        log::debug!("WebSocket handshake {}, response: {:?}", uri, response);
        let (sink, stream) = ws_stream.split();
        Ok(Self {
            sender: WsSender::new(sink),
            receiver: WsReceiver::new(stream),
            manager: TaskManager::new(max_capacity_per_subscription),
        })
    }

    /// Convert self into a spawnable runtime task that processes message sent from the frontend and
    /// received from backend.
    pub(crate) async fn into_task(self, from_front: mpsc::Receiver<ToBackTaskMessage>) {
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
                    Some(msg) => handle_from_front_message(msg, &mut manager, &mut sender).await,
                    None => {
                        log::error!("[backend] Frontend channel dropped; terminate client");
                        break;
                    }
                },
                msg = from_back.next() => match msg {
                    Some(Ok(msg)) => if let Err(err) = handle_from_back_message(msg, &mut manager, &mut sender).await {
                        log::error!("[backend] Handle websocket message error: {}; terminate client", err);
                        break;
                    }
                    Some(Err(err)) => {
                        log::error!("[backend] Receive websocket message error: {}; terminate client", err);
                        break;
                    }
                    None => {
                        log::error!("[backend] Backend channel dropped; terminate client");
                        break;
                    }
                },
            }
        }
    }
}

async fn handle_from_front_message(msg: ToBackTaskMessage, manager: &mut TaskManager, sender: &mut WsSender) {
    match msg {
        ToBackTaskMessage::Request {
            method,
            params,
            send_back,
        } => match sender.send_request(method, params).await {
            Ok(req_id) => {
                if let Err(send_back) = manager.insert_pending_method_call(req_id, send_back) {
                    send_back
                        .send(Err(WsClientError::DuplicateRequestId))
                        .expect("Send request error back");
                }
            }
            Err(err) => {
                log::warn!("[backend] Send request error: {}", err);
                send_back
                    .send(Err(WsClientError::WebSocket(err)))
                    .expect("Send request error back");
            }
        },
        ToBackTaskMessage::BatchRequest { batch, send_back } => match sender.send_batch_request(batch).await {
            Ok(req_ids) => {
                let min_request_id = req_ids.into_iter().min().expect("must have one");
                if let Err(send_back) = manager.insert_pending_batch_method_call(min_request_id, send_back) {
                    send_back
                        .send(Err(WsClientError::DuplicateRequestId))
                        .expect("Send batch request error back");
                }
            }
            Err(err) => {
                log::warn!("[backend] Send a batch of requests error: {}", err);
                send_back
                    .send(Err(WsClientError::WebSocket(err)))
                    .expect("Send batch request error back");
            }
        },
        ToBackTaskMessage::Subscribe {
            subscribe_method,
            unsubscribe_method,
            params,
            send_back,
        } => match sender.start_subscription(subscribe_method, params).await {
            Ok(req_id) => {
                if let Err(send_back) = manager.insert_pending_subscription(req_id, send_back, unsubscribe_method) {
                    send_back
                        .send(Err(WsClientError::DuplicateRequestId))
                        .expect("Send subscription request error back");
                }
            }
            Err(err) => {
                log::warn!("[backend] Send subscription request error: {}", err);
                send_back
                    .send(Err(WsClientError::WebSocket(err)))
                    .expect("Send subscription request error back");
            }
        },
        ToBackTaskMessage::SubscriptionClosed(subscription_id) => {
            log::debug!("[backend] Close subscription: id={:?}", subscription_id);
            // NOTE: The subscription may have been closed earlier if the channel was full or disconnected.
            if let Some(request_id) = manager.get_request_id_by(&subscription_id) {
                if let Some((_sink, unsubscribe_method)) =
                    manager.remove_active_subscription(request_id, subscription_id.clone())
                {
                    if let Err(err) = sender.stop_subscription(unsubscribe_method, subscription_id).await {
                        log::error!("[backend] Send unsubscription error: {}", err);
                    }
                }
            }
        }
    }
}

async fn handle_from_back_message(
    msg: Message,
    manager: &mut TaskManager,
    sender: &mut WsSender,
) -> Result<(), WsClientError> {
    match msg {
        Message::Text(msg) => {
            if let Ok(response) = serde_json::from_str::<Response>(&msg) {
                handle_response_message(response, manager)?
            } else if let Ok(notification) = serde_json::from_str::<SubscriptionNotification>(&msg) {
                handle_subscription_notification_message(notification, manager);
            } else {
                log::warn!("[backend] Ignore unknown websocket text message: {}", msg);
            }
        }
        Message::Binary(msg) => log::warn!("[backend] Ignore `Binary` message: {:?}", msg),
        Message::Ping(msg) => {
            log::debug!("[backend] Receive `Ping` message: {:?}", msg);
            log::debug!("[backend] Send `Pong` message back, message: {:?}", msg);
            sender.send_message(Message::Pong(msg)).await?;
        }
        Message::Pong(msg) => log::debug!("[backend] Receive `Pong` message: {:?}", msg),
        Message::Close(msg) => {
            log::error!("[backend] Receive `Close` message: {:?}; terminate client", msg);
            return Err(WsClientError::WebSocket(WsError::ConnectionClosed));
        }
    }
    Ok(())
}

fn handle_response_message(response: Response, manager: &mut TaskManager) -> Result<(), WsClientError> {
    match response {
        Response::Single(output) => handle_single_output(output, manager),
        Response::Batch(outputs) => handle_batch_output(outputs, manager),
    }
}

fn handle_single_output(output: Output, manager: &mut TaskManager) -> Result<(), WsClientError> {
    let response_id = response_id_of(&output)?;
    match manager.request_status(&response_id) {
        RequestStatus::PendingMethodCall => {
            log::debug!("[backend] Handle single response of method call: id={}", response_id);
            let send_back = manager
                .complete_pending_method_call(response_id)
                .ok_or(WsClientError::InvalidRequestId)?;
            send_back.send(Ok(output)).expect("Send single response back");
            Ok(())
        }
        RequestStatus::PendingSubscription => {
            log::debug!("[backend] Handle response of subscription request: id={}", response_id);
            let (send_back, unsubscribe_method) = manager
                .complete_pending_subscription(response_id)
                .ok_or(WsClientError::InvalidRequestId)?;
            let subscription_id = match output {
                Output::Success(success) => match serde_json::from_value::<Id>(success.result) {
                    Ok(id) => id,
                    Err(err) => {
                        send_back
                            .send(Err(WsClientError::Json(err)))
                            .expect("Send response error back");
                        return Ok(());
                    }
                },
                Output::Failure(_) => {
                    send_back
                        .send(Err(WsClientError::InvalidSubscriptionId))
                        .expect("Send response error back");
                    return Ok(());
                }
            };

            let (subscribe_tx, subscribe_rx) = mpsc::channel(manager.max_capacity_per_subscription);
            if manager
                .insert_active_subscription(response_id, subscription_id.clone(), subscribe_tx, unsubscribe_method)
                .is_ok()
            {
                send_back
                    .send(Ok((subscription_id, subscribe_rx)))
                    .expect("Send subscription stream back");
                Ok(())
            } else {
                send_back
                    .send(Err(WsClientError::InvalidSubscriptionId))
                    .expect("Send subscription error back");
                Ok(())
            }
        }
        RequestStatus::ActiveSubscription | RequestStatus::PendingBatchMethodCall | RequestStatus::Invalid => {
            Err(WsClientError::InvalidRequestId)
        }
    }
}

fn response_id_of(output: &Output) -> Result<u64, WsClientError> {
    Ok(*output
        .id()
        .ok_or(WsClientError::InvalidRequestId)?
        .as_number()
        .expect("Response ID must be number"))
}

fn handle_batch_output(outputs: Vec<Output>, manager: &mut TaskManager) -> Result<(), WsClientError> {
    let (min_response_id, max_response_id) = response_id_range_of(&outputs)?;
    // use the min id of batch request for managing task
    match manager.request_status(&min_response_id) {
        RequestStatus::PendingBatchMethodCall => {
            log::debug!(
                "[backend] Handle batch response of batch request: id=({}~{})",
                min_response_id,
                max_response_id
            );
            let send_back = manager
                .complete_pending_batch_method_call(min_response_id)
                .ok_or(WsClientError::InvalidRequestId)?;
            send_back.send(Ok(outputs)).expect("Send batch response back");
            Ok(())
        }
        RequestStatus::PendingMethodCall
        | RequestStatus::PendingSubscription
        | RequestStatus::ActiveSubscription
        | RequestStatus::Invalid => Err(WsClientError::InvalidRequestId),
    }
}

fn response_id_range_of(outputs: &[Output]) -> Result<(u64, u64), WsClientError> {
    assert!(!outputs.is_empty());
    let (mut min, mut max) = (u64::max_value(), u64::min_value());
    for output in outputs {
        let id = *output
            .id()
            .ok_or(WsClientError::InvalidRequestId)?
            .as_number()
            .expect("Response ID must be number");
        min = std::cmp::min(id, min);
        max = std::cmp::max(id, max);
    }
    Ok((min, max))
}

fn handle_subscription_notification_message(notification: SubscriptionNotification, manager: &mut TaskManager) {
    let subscription_id = notification.params.subscription.clone();
    let request_id = match manager.get_request_id_by(&subscription_id) {
        Some(id) => id,
        None => {
            log::error!(
                "[backend] Task manager cannot find subscription: id={:?}",
                subscription_id
            );
            return;
        }
    };
    match manager.as_active_subscription_mut(&request_id) {
        Some(send_back) => {
            if let Err(err) = send_back.try_send(notification) {
                log::error!("[backend] Dropping subscription: id={:?}: {}", subscription_id, err);
                manager
                    .remove_active_subscription(request_id, subscription_id)
                    .expect("kind is ActiveSubscription; qed");
            }
        }
        None => log::error!(
            "[backend] Subscription id ({:?}) is not an active subscription",
            subscription_id
        ),
    }
}
