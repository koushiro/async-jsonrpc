use futures::channel::mpsc::UnboundedSender;

use crate::types::Request;

/// Subscription ID.
pub type SubscriptionId = usize;

/// Subscription.
pub type Subscription = UnboundedSender<Request>;
