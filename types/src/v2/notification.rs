#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};
use core::fmt;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

use crate::v2::{Id, Params, ParamsRef, Version};

/// Represents JSON-RPC 1.0 batch notification.
pub type BatchNotificationRef<'a> = Vec<NotificationRef<'a>>;

/// Represents JSON-RPC 2.0 request which is a notification.
///
/// A Request object that is a Notification signifies the Client's lack of interest in the
/// corresponding Response object, and as such no Response object needs to be returned to the client.
/// As such, the Client would not be aware of any errors (like e.g. "Invalid params","Internal error").
///
/// The Server MUST NOT reply to a Notification, including those that are within a batch request.
#[derive(Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NotificationRef<'a> {
    /// A String specifying the version of the JSON-RPC protocol.
    pub jsonrpc: Version,
    /// A String containing the name of the method to be invoked.
    ///
    /// Method names that begin with the word rpc followed by a period character (U+002E or ASCII 46)
    /// are reserved for rpc-internal methods and extensions and MUST NOT be used for anything else.
    pub method: &'a str,
    /// A Structured value that holds the parameter values to be used
    /// during the invocation of the method. This member MAY be omitted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<ParamsRef<'a>>,
}

impl<'a> fmt::Display for NotificationRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let json = serde_json::to_string(self).expect("`NotificationRef` is serializable");
        write!(f, "{}", json)
    }
}

impl<'a> PartialEq<Notification> for NotificationRef<'a> {
    fn eq(&self, other: &Notification) -> bool {
        self.method.eq(&other.method) && self.params.eq(&other.params.as_ref().map(|params| params.as_ref()))
    }
}

impl<'a> NotificationRef<'a> {
    /// Creates a JSON-RPC 2.0 request which is a notification.
    pub fn new(method: &'a str, params: Option<ParamsRef<'a>>) -> Self {
        Self {
            jsonrpc: Version::V2_0,
            method,
            params,
        }
    }

    /// Converts the reference into the owned type.
    pub fn to_owned(&self) -> Notification {
        Notification {
            jsonrpc: self.jsonrpc,
            method: self.method.into(),
            params: self.params.as_ref().map(|params| params.to_owned()),
        }
    }
}

// ################################################################################################

/// Represents JSON-RPC 1.0 batch notification.
pub type BatchNotification = Vec<Notification>;

/// Represents JSON-RPC 2.0 request which is a notification.
///
/// A Request object that is a Notification signifies the Client's lack of interest in the
/// corresponding Response object, and as such no Response object needs to be returned to the client.
/// As such, the Client would not be aware of any errors (like e.g. "Invalid params","Internal error").
///
/// The Server MUST NOT reply to a Notification, including those that are within a batch request.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Notification {
    /// A String specifying the version of the JSON-RPC protocol.
    pub jsonrpc: Version,
    /// A String containing the name of the method to be invoked.
    ///
    /// Method names that begin with the word rpc followed by a period character (U+002E or ASCII 46)
    /// are reserved for rpc-internal methods and extensions and MUST NOT be used for anything else.
    pub method: String,
    /// A Structured value that holds the parameter values to be used
    /// during the invocation of the method. This member MAY be omitted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Params>,
}

impl fmt::Display for Notification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let json = serde_json::to_string(self).expect("`Notification` is serializable");
        write!(f, "{}", json)
    }
}

impl<'a> PartialEq<NotificationRef<'a>> for Notification {
    fn eq(&self, other: &NotificationRef<'a>) -> bool {
        self.method.eq(other.method) && self.params.as_ref().map(|params| params.as_ref()).eq(&other.params)
    }
}

impl Notification {
    /// Creates a JSON-RPC 2.0 request which is a notification.
    pub fn new<M: Into<String>>(method: M, params: Option<Params>) -> Self {
        Self {
            jsonrpc: Version::V2_0,
            method: method.into(),
            params,
        }
    }

    /// Borrows from an owned value.
    pub fn as_ref(&self) -> NotificationRef<'_> {
        NotificationRef {
            jsonrpc: self.jsonrpc,
            method: &self.method,
            params: self.params.as_ref().map(|params| params.as_ref()),
        }
    }
}

// ################################################################################################

/// Parameters of the subscription notification.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubscriptionNotificationParams<T = Value> {
    /// Subscription id, as communicated during the subscription.
    pub subscription: Id,
    /// Actual data that the server wants to communicate to the client.
    pub result: T,
}

impl<T: Serialize + DeserializeOwned> SubscriptionNotificationParams<T> {
    /// Creates a JSON-RPC 2.0 notification parameter.
    pub fn new(id: Id, result: T) -> Self {
        Self {
            subscription: id,
            result,
        }
    }
}

/// Server notification about something the client is subscribed to.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubscriptionNotification<T = Value> {
    /// A String specifying the version of the JSON-RPC protocol.
    pub jsonrpc: Version,
    /// A String containing the name of the method that was used for the subscription.
    pub method: String,
    /// Parameters of the subscription notification.
    pub params: SubscriptionNotificationParams<T>,
}

impl<T: Serialize> fmt::Display for SubscriptionNotification<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let json = serde_json::to_string(self).expect("`SubscriptionNotification` is serializable");
        write!(f, "{}", json)
    }
}

impl<T: Serialize + DeserializeOwned> SubscriptionNotification<T> {
    /// Creates a JSON-RPC 2.0 notification which is a subscription notification.
    pub fn new<M: Into<String>>(method: M, params: SubscriptionNotificationParams<T>) -> Self {
        Self {
            jsonrpc: Version::V2_0,
            method: method.into(),
            params,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn notification_cases() -> Vec<(Notification, &'static str)> {
        vec![
            (
                // JSON-RPC 2.0 notification
                Notification::new("foo", Some(Params::Array(vec![Value::from(1), Value::Bool(true)]))),
                r#"{"jsonrpc":"2.0","method":"foo","params":[1,true]}"#,
            ),
            (
                // JSON-RPC 2.0 notification with an empty array parameters
                Notification::new("foo", Some(Params::Array(vec![]))),
                r#"{"jsonrpc":"2.0","method":"foo","params":[]}"#,
            ),
            (
                // JSON-RPC 2.0 notification without parameters
                Notification::new("foo", None),
                r#"{"jsonrpc":"2.0","method":"foo"}"#,
            ),
        ]
    }

    #[test]
    fn notification_serialization() {
        for (notification, expect) in notification_cases() {
            let ser = serde_json::to_string(&notification).unwrap();
            assert_eq!(ser, expect);
            let de = serde_json::from_str::<Notification>(expect).unwrap();
            assert_eq!(de, notification);
        }

        // JSON-RPC 2.0 valid notification
        let valid_cases = vec![
            r#"{"jsonrpc":"2.0","method":"foo","params":[1,true]}"#,
            r#"{"jsonrpc":"2.0","method":"foo","params":[]}"#,
            r#"{"jsonrpc":"2.0","method":"foo"}"#,
        ];
        for case in valid_cases {
            let request = serde_json::from_str::<Notification>(case);
            assert!(request.is_ok());
        }

        // JSON-RPC 2.0 invalid notification
        let invalid_cases = vec![
            r#"{"jsonrpc":"2.0","method":"foo","params":[1,true],"id":1,"unknown":[]}"#,
            r#"{"jsonrpc":"2.0"`,"method":"foo","params":[1,true],"id":1.2}"#,
            r#"{"jsonrpc":"2.0","method":"foo","params":[1,true],"id":null,"unknown":[]}"#,
            r#"{"jsonrpc":"2.0","method":"foo","params":[1,true],"id":null}"#,
            r#"{"jsonrpc":"2.0","method":"foo","params":[1,true],"unknown":[]}"#,
            r#"{"jsonrpc":"2.0","method":"foo","unknown":[]}"#,
            r#"{"jsonrpc":"2.0","unknown":[]}"#,
        ];
        for case in invalid_cases {
            let request = serde_json::from_str::<Notification>(case);
            assert!(request.is_err());
        }
    }
}
