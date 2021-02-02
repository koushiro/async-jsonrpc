use std::collections::hash_map::{Entry, HashMap};

use futures::channel::{mpsc, oneshot};
use jsonrpc_types::*;

use crate::error::Result;

type PendingMethodCall = oneshot::Sender<Result<Response>>;
type PendingSubscription = oneshot::Sender<Result<mpsc::Receiver<SubscriptionNotification>>>;
type SubscriptionSink = mpsc::Sender<MethodCall>;
type UnsubscribeMethod = String;

enum RequestKind {
    PendingMethodCall(PendingMethodCall),
    PendingSubscription((PendingSubscription, UnsubscribeMethod)),
    Subscription((SubscriptionSink, UnsubscribeMethod)),
}

pub enum RequestStatus {
    /// The method call is waiting for a response
    PendingMethodCall,
    /// The subscription is waiting for a response to become an active subscription.
    PendingSubscription,
    /// An active subscription.
    Subscription,
    /// Invalid request ID.
    Invalid,
}

/// Manages JSON-RPC 2.0 method calls and subscriptions.
pub struct TaskManager {
    requests: HashMap<u64, RequestKind>,
    subscriptions: HashMap<Id, u64>,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            requests: HashMap::new(),
            subscriptions: HashMap::new(),
        }
    }

    /// Tries to insert a new pending method call.
    pub fn insert_pending_method_call(
        &mut self,
        request_id: u64,
        send_back: PendingMethodCall,
    ) -> Result<(), PendingMethodCall> {
        if let Entry::Vacant(v) = self.requests.entry(request_id) {
            v.insert(RequestKind::PendingMethodCall(send_back));
            Ok(())
        } else {
            Err(send_back)
        }
    }

    /// Tries to insert a new pending subscription.
    pub fn insert_pending_subscription(
        &mut self,
        request_id: u64,
        send_back: PendingSubscription,
        unsubscribe_method: UnsubscribeMethod,
    ) -> Result<(), PendingSubscription> {
        if let Entry::Vacant(v) = self.requests.entry(request_id) {
            v.insert(RequestKind::PendingSubscription((
                send_back,
                unsubscribe_method,
            )));
            Ok(())
        } else {
            Err(send_back)
        }
    }

    /// Tries to insert a new active subscription.
    pub fn insert_subscription(
        &mut self,
        request_id: u64,
        subscription_id: Id,
        send_back: SubscriptionSink,
        unsubscribe_method: UnsubscribeMethod,
    ) -> Result<(), SubscriptionSink> {
        match (
            self.requests.entry(request_id),
            self.subscriptions.entry(subscription_id),
        ) {
            (Entry::Vacant(request), Entry::Vacant(subscription)) => {
                request.insert(RequestKind::Subscription((send_back, unsubscribe_method)));
                subscription.insert(request_id);
                Ok(())
            }
            _ => Err(send_back),
        }
    }

    /// Tries to complete a pending method call.
    pub fn complete_pending_method_call(&mut self, request_id: u64) -> Option<PendingMethodCall> {
        match self.requests.entry(request_id) {
            Entry::Occupied(request)
                if matches!(request.get(), RequestKind::PendingMethodCall(_)) =>
            {
                if let (_req_id, RequestKind::PendingMethodCall(send_back)) = request.remove_entry()
                {
                    Some(send_back)
                } else {
                    unreachable!()
                }
            }
            _ => None,
        }
    }

    /// Tries to complete a pending subscription.
    pub fn complete_pending_subscription(
        &mut self,
        request_id: u64,
    ) -> Option<(PendingSubscription, UnsubscribeMethod)> {
        match self.requests.entry(request_id) {
            Entry::Occupied(request)
                if matches!(request.get(), RequestKind::PendingSubscription(_)) =>
            {
                if let (_req_id, RequestKind::PendingSubscription(send_back)) =
                    request.remove_entry()
                {
                    Some(send_back)
                } else {
                    unreachable!()
                }
            }
            _ => None,
        }
    }

    /// Tries to remove a subscription.
    pub fn remove_subscription(
        &mut self,
        request_id: u64,
        subscription_id: Id,
    ) -> Option<(SubscriptionSink, UnsubscribeMethod)> {
        match (
            self.requests.entry(request_id),
            self.subscriptions.entry(subscription_id),
        ) {
            (Entry::Occupied(requests), Entry::Occupied(subscriptions)) => {
                let (_req_id, kind) = requests.remove_entry();
                let _sub_id = subscriptions.remove_entry();
                if let RequestKind::Subscription(send_back) = kind {
                    Some(send_back)
                } else {
                    unreachable!()
                }
            }
            _ => None,
        }
    }

    /// Returns the status of a request ID.
    pub fn request_status(&mut self, request_id: u64) -> RequestStatus {
        self.requests
            .get(&request_id)
            .map_or(RequestStatus::Invalid, |kind| match kind {
                RequestKind::PendingMethodCall(_) => RequestStatus::PendingMethodCall,
                RequestKind::PendingSubscription(_) => RequestStatus::PendingSubscription,
                RequestKind::Subscription(_) => RequestStatus::Subscription,
            })
    }
}
