use std::collections::hash_map::{Entry, HashMap};

use futures::channel::{mpsc, oneshot};
use jsonrpc_types::*;

use crate::error::WsClientError;

type PendingMethodCall = oneshot::Sender<Result<Output, WsClientError>>;
type PendingBatchMethodCall = oneshot::Sender<Result<Vec<Output>, WsClientError>>;
type PendingSubscription = oneshot::Sender<Result<(Id, mpsc::Receiver<SubscriptionNotification>), WsClientError>>;
type ActiveSubscription = mpsc::Sender<SubscriptionNotification>;
type UnsubscribeMethod = String;

#[derive(Debug)]
enum RequestKind {
    PendingMethodCall(PendingMethodCall),
    PendingBatchMethodCall(PendingBatchMethodCall),
    PendingSubscription((PendingSubscription, UnsubscribeMethod)),
    ActiveSubscription((ActiveSubscription, UnsubscribeMethod)),
}

pub enum RequestStatus {
    /// The method call is waiting for a response
    PendingMethodCall,
    /// The batch of method calls is waiting for batch of responses.
    PendingBatchMethodCall,
    /// The subscription is waiting for a response to become an active subscription.
    PendingSubscription,
    /// An active subscription.
    ActiveSubscription,
    /// Invalid request ID.
    Invalid,
}

/// Manages JSON-RPC 2.0 method calls and subscriptions.
#[derive(Debug)]
pub struct TaskManager {
    /// Requests that are waiting for response from the server.
    requests: HashMap<u64, RequestKind>,
    /// Helper to find a request ID by subscription ID instead of looking through all requests.
    subscriptions: HashMap<Id, u64>,
    /// Max capacity of every subscription channel.
    pub(crate) max_capacity_per_subscription: usize,
}

impl TaskManager {
    pub fn new(max_capacity_per_subscription: usize) -> Self {
        Self {
            requests: HashMap::new(),
            subscriptions: HashMap::new(),
            max_capacity_per_subscription,
        }
    }

    /// Tries to insert a new pending method call into manager.
    pub fn insert_pending_method_call(
        &mut self,
        request_id: u64,
        send_back: PendingMethodCall,
    ) -> Result<(), PendingMethodCall> {
        match self.requests.entry(request_id) {
            Entry::Vacant(request) => {
                request.insert(RequestKind::PendingMethodCall(send_back));
                Ok(())
            }
            // Duplicate request ID.
            Entry::Occupied(_) => Err(send_back),
        }
    }

    /// Tries to complete a pending method call from manager.
    pub fn complete_pending_method_call(&mut self, request_id: u64) -> Option<PendingMethodCall> {
        match self.requests.entry(request_id) {
            Entry::Occupied(request) if matches!(request.get(), RequestKind::PendingMethodCall(_)) => {
                if let (_req_id, RequestKind::PendingMethodCall(send_back)) = request.remove_entry() {
                    Some(send_back)
                } else {
                    unreachable!("Kind must be PendingMethodCall; qed");
                }
            }
            _ => None,
        }
    }

    /// Tries to insert a new pending method call into manager.
    pub fn insert_pending_batch_method_call(
        &mut self,
        min_request_id: u64,
        send_back: PendingBatchMethodCall,
    ) -> Result<(), PendingBatchMethodCall> {
        match self.requests.entry(min_request_id) {
            Entry::Vacant(request) => {
                request.insert(RequestKind::PendingBatchMethodCall(send_back));
                Ok(())
            }
            // Duplicate request ID.
            Entry::Occupied(_) => Err(send_back),
        }
    }

    /// Tries to complete a pending batch method call from manager.
    pub fn complete_pending_batch_method_call(&mut self, min_request_id: u64) -> Option<PendingBatchMethodCall> {
        match self.requests.entry(min_request_id) {
            Entry::Occupied(request) if matches!(request.get(), RequestKind::PendingBatchMethodCall(_)) => {
                if let (_min_req_id, RequestKind::PendingBatchMethodCall(send_back)) = request.remove_entry() {
                    Some(send_back)
                } else {
                    unreachable!("Kind must be PendingMethodCall; qed");
                }
            }
            _ => None,
        }
    }

    /// Tries to insert a new pending subscription into manager.
    pub fn insert_pending_subscription(
        &mut self,
        request_id: u64,
        send_back: PendingSubscription,
        unsubscribe_method: UnsubscribeMethod,
    ) -> Result<(), PendingSubscription> {
        match self.requests.entry(request_id) {
            Entry::Vacant(request) => {
                request.insert(RequestKind::PendingSubscription((send_back, unsubscribe_method)));
                Ok(())
            }
            // Duplicate request ID.
            Entry::Occupied(_) => Err(send_back),
        }
    }

    /// Tries to complete a pending subscription from manager.
    pub fn complete_pending_subscription(
        &mut self,
        request_id: u64,
    ) -> Option<(PendingSubscription, UnsubscribeMethod)> {
        match self.requests.entry(request_id) {
            Entry::Occupied(request) if matches!(request.get(), RequestKind::PendingSubscription(_)) => {
                if let (_id, RequestKind::PendingSubscription(send_back)) = request.remove_entry() {
                    Some(send_back)
                } else {
                    unreachable!("Kind must be PendingSubscription; qed");
                }
            }
            _ => None,
        }
    }

    /// Tries to insert a new active subscription into manager.
    pub fn insert_active_subscription(
        &mut self,
        request_id: u64,
        subscription_id: Id,
        send_back: ActiveSubscription,
        unsubscribe_method: UnsubscribeMethod,
    ) -> Result<(), ActiveSubscription> {
        match (
            self.requests.entry(request_id),
            self.subscriptions.entry(subscription_id),
        ) {
            (Entry::Vacant(request), Entry::Vacant(subscription)) => {
                request.insert(RequestKind::ActiveSubscription((send_back, unsubscribe_method)));
                subscription.insert(request_id);
                Ok(())
            }
            // Duplicate request ID or subscription ID.
            _ => Err(send_back),
        }
    }

    /// Tries to remove an active subscription from manager.
    pub fn remove_active_subscription(
        &mut self,
        request_id: u64,
        subscription_id: Id,
    ) -> Option<(ActiveSubscription, UnsubscribeMethod)> {
        match (
            self.requests.entry(request_id),
            self.subscriptions.entry(subscription_id),
        ) {
            (Entry::Occupied(request), Entry::Occupied(subscription)) => {
                let (_req_id, kind) = request.remove_entry();
                let (_sub_id, _req_id) = subscription.remove_entry();
                if let RequestKind::ActiveSubscription(send_back) = kind {
                    Some(send_back)
                } else {
                    unreachable!("Kind must be ActiveSubscription; qed");
                }
            }
            _ => None,
        }
    }

    /// Reverse lookup to get the request ID by a subscription ID.
    pub fn get_request_id_by(&self, subscription_id: &Id) -> Option<u64> {
        self.subscriptions.get(subscription_id).copied()
    }

    /// Returns the status of a request ID.
    pub fn request_status(&mut self, request_id: &u64) -> RequestStatus {
        self.requests
            .get(request_id)
            .map_or(RequestStatus::Invalid, |kind| match kind {
                RequestKind::PendingMethodCall(_) => RequestStatus::PendingMethodCall,
                RequestKind::PendingBatchMethodCall(_) => RequestStatus::PendingBatchMethodCall,
                RequestKind::PendingSubscription(_) => RequestStatus::PendingSubscription,
                RequestKind::ActiveSubscription(_) => RequestStatus::ActiveSubscription,
            })
    }

    /// Gets a mutable reference to active subscription sink to send messages back to
    /// the subscription channel.
    pub fn as_active_subscription_mut(&mut self, request_id: &u64) -> Option<&mut ActiveSubscription> {
        let kind = self.requests.get_mut(request_id);
        if let Some(RequestKind::ActiveSubscription((sink, _))) = kind {
            Some(sink)
        } else {
            None
        }
    }
}
