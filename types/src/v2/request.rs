use std::fmt;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{from_value, Map, Value};

use crate::{error::Error, id::Id, v2::version::Version};

/// Represents JSON-RPC 2.0 request parameters.
///
/// If present, parameters for the rpc call MUST be provided as a Structured value.
/// Either by-position through an Array or by-name through an Object.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Params {
    /// Array of values
    Array(Vec<Value>),
    /// Map of values
    Map(Map<String, Value>),
}

impl Default for Params {
    fn default() -> Self {
        Params::Array(vec![])
    }
}

impl fmt::Display for Params {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let json = serde_json::to_string(self).expect("`Params` is serializable");
        write!(f, "{}", json)
    }
}

impl Params {
    /// Parses incoming `Params` into expected types.
    pub fn parse<D>(self) -> Result<D, Error>
    where
        D: DeserializeOwned,
    {
        let value = self.into();
        from_value(value).map_err(Error::invalid_params)
    }

    /// Checks if the parameters is an empty array of objects.
    pub fn is_empty_array(&self) -> bool {
        matches!(self, Params::Array(array) if array.is_empty())
    }

    /// Checks if the parameters is an array of objects.
    pub fn is_array(&self) -> bool {
        matches!(self, Params::Array(_))
    }

    /// Checks if the parameters is a map of objects.
    pub fn is_map(&self) -> bool {
        matches!(self, Params::Map(_))
    }
}

impl From<Params> for Value {
    fn from(params: Params) -> Value {
        match params {
            Params::Array(array) => Value::Array(array),
            Params::Map(object) => Value::Object(object),
        }
    }
}

/// Represents JSON-RPC 2.0 request which is a method call.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MethodCall {
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
    /// An identifier established by the Client.
    /// If it is not included it is assumed to be a notification.
    pub id: Id,
}

impl fmt::Display for MethodCall {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let json = serde_json::to_string(self).expect("`MethodCall` is serializable");
        write!(f, "{}", json)
    }
}

impl MethodCall {
    /// Creates a JSON-RPC 2.0 request which is a method call.
    pub fn new<M: Into<String>>(method: M, params: Option<Params>, id: Id) -> Self {
        Self {
            jsonrpc: Version::V2_0,
            method: method.into(),
            params,
            id,
        }
    }
}

/// Represents JSON-RPC 2.0 request which is a notification.
///
/// A Request object that is a Notification signifies the Client's lack of interest in the
/// corresponding Response object, and as such no Response object needs to be returned to the client.
/// As such, the Client would not be aware of any errors (like e.g. "Invalid params","Internal error").
///
/// The Server MUST NOT reply to a Notification, including those that are within a batch request.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
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

impl Notification {
    /// Creates a JSON-RPC 2.0 request which is a notification.
    pub fn new<M: Into<String>>(method: M, params: Option<Params>) -> Self {
        Self {
            jsonrpc: Version::V2_0,
            method: method.into(),
            params,
        }
    }
}

/// Parameters of the subscription notification.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
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
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
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

/// Represents single JSON-RPC 2.0 call.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum Call {
    /// Call method
    MethodCall(MethodCall),
    /// Fire notification
    Notification(Notification),
}

impl fmt::Display for Call {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let json = serde_json::to_string(self).expect("`Call` is serializable");
        write!(f, "{}", json)
    }
}

impl Call {
    /// Returns the method of the request call.
    pub fn method(&self) -> &str {
        match self {
            Self::MethodCall(call) => &call.method,
            Self::Notification(notification) => &notification.method,
        }
    }

    /// Returns the params of the request call.
    pub fn params(&self) -> &Option<Params> {
        match self {
            Self::MethodCall(call) => &call.params,
            Self::Notification(notification) => &notification.params,
        }
    }

    /// Returns the id of the request call.
    pub fn id(&self) -> Option<Id> {
        match self {
            Self::MethodCall(call) => Some(call.id.clone()),
            Self::Notification(_notification) => None,
        }
    }
}

impl From<MethodCall> for Call {
    fn from(call: MethodCall) -> Self {
        Self::MethodCall(call)
    }
}

impl From<Notification> for Call {
    fn from(notify: Notification) -> Self {
        Self::Notification(notify)
    }
}

/// JSON-RPC 2.0 Request object.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum Request {
    /// Single call
    Single(Call),
    /// Batch of calls
    Batch(Vec<Call>),
}

impl fmt::Display for Request {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let json = serde_json::to_string(self).expect("`Request` is serializable");
        write!(f, "{}", json)
    }
}

/// JSON-RPC 2.0 Request object (only for method call).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum MethodCallRequest {
    /// Single method call
    Single(MethodCall),
    /// Batch of method calls
    Batch(Vec<MethodCall>),
}

impl From<MethodCall> for MethodCallRequest {
    fn from(call: MethodCall) -> Self {
        Self::Single(call)
    }
}

impl From<Vec<MethodCall>> for MethodCallRequest {
    fn from(calls: Vec<MethodCall>) -> Self {
        Self::Batch(calls)
    }
}

impl fmt::Display for MethodCallRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let json = serde_json::to_string(self).expect("`MethodCallRequest` is serializable");
        write!(f, "{}", json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn params_serialization() {
        let array = vec![Value::from(1), Value::Bool(true)];
        let params = Params::Array(array.clone());
        assert_eq!(serde_json::to_string(&params).unwrap(), r#"[1,true]"#);
        assert_eq!(serde_json::from_str::<Params>(r#"[1,true]"#).unwrap(), params);

        let object = {
            let mut map = Map::new();
            map.insert("key".into(), Value::String("value".into()));
            map
        };
        let params = Params::Map(object.clone());
        assert_eq!(serde_json::to_string(&params).unwrap(), r#"{"key":"value"}"#);
        assert_eq!(serde_json::from_str::<Params>(r#"{"key":"value"}"#).unwrap(), params);

        let params = Params::Array(vec![
            Value::Null,
            Value::Bool(true),
            Value::from(-1),
            Value::from(1),
            Value::from(1.2),
            Value::String("hello".to_string()),
            Value::Array(vec![]),
            Value::Array(array),
            Value::Object(object),
        ]);
        assert_eq!(
            serde_json::to_string(&params).unwrap(),
            r#"[null,true,-1,1,1.2,"hello",[],[1,true],{"key":"value"}]"#
        );
        assert_eq!(
            serde_json::from_str::<Params>(r#"[null,true,-1,1,1.2,"hello",[],[1,true],{"key":"value"}]"#).unwrap(),
            params
        );
    }

    #[test]
    fn single_param_parsed_as_tuple() {
        let params: (u64,) = Params::Array(vec![Value::from(1)]).parse().unwrap();
        assert_eq!(params, (1,));
    }

    #[test]
    fn invalid_params() {
        let params = serde_json::from_str::<Params>("[1,true]").unwrap();
        assert_eq!(
            params.parse::<(u8, bool, String)>().unwrap_err(),
            Error::invalid_params("invalid length 2, expected a tuple of size 3")
        );
    }

    fn method_call_cases() -> Vec<(MethodCall, &'static str)> {
        vec![
            (
                // JSON-RPC 2.0 request method call
                MethodCall {
                    jsonrpc: Version::V2_0,
                    method: "foo".to_string(),
                    params: Some(Params::Array(vec![Value::from(1), Value::Bool(true)])),
                    id: Id::Num(1),
                },
                r#"{"jsonrpc":"2.0","method":"foo","params":[1,true],"id":1}"#,
            ),
            (
                // JSON-RPC 2.0 request method call with an empty array parameters
                MethodCall {
                    jsonrpc: Version::V2_0,
                    method: "foo".to_string(),
                    params: Some(Params::Array(vec![])),
                    id: Id::Num(1),
                },
                r#"{"jsonrpc":"2.0","method":"foo","params":[],"id":1}"#,
            ),
            (
                // JSON-RPC 2.0 request method call without parameters
                MethodCall {
                    jsonrpc: Version::V2_0,
                    method: "foo".to_string(),
                    params: None,
                    id: Id::Num(1),
                },
                r#"{"jsonrpc":"2.0","method":"foo","id":1}"#,
            ),
        ]
    }

    fn notification_cases() -> Vec<(Notification, &'static str)> {
        vec![
            (
                // JSON-RPC 2.0 request notification
                Notification {
                    jsonrpc: Version::V2_0,
                    method: "foo".to_string(),
                    params: Some(Params::Array(vec![Value::from(1), Value::Bool(true)])),
                },
                r#"{"jsonrpc":"2.0","method":"foo","params":[1,true]}"#,
            ),
            (
                // JSON-RPC 2.0 request method call with an empty array parameters
                Notification {
                    jsonrpc: Version::V2_0,
                    method: "foo".to_string(),
                    params: Some(Params::Array(vec![])),
                },
                r#"{"jsonrpc":"2.0","method":"foo","params":[]}"#,
            ),
            (
                // JSON-RPC 2.0 request notification without parameters
                Notification {
                    jsonrpc: Version::V2_0,
                    method: "foo".to_string(),
                    params: None,
                },
                r#"{"jsonrpc":"2.0","method":"foo"}"#,
            ),
        ]
    }

    #[test]
    fn method_call_serialization() {
        for (method_call, expect) in method_call_cases() {
            let ser = serde_json::to_string(&method_call).unwrap();
            assert_eq!(ser, expect);
            let de = serde_json::from_str::<MethodCall>(expect).unwrap();
            assert_eq!(de, method_call);
        }
    }

    #[test]
    fn notification_serialization() {
        for (notification, expect) in notification_cases() {
            let ser = serde_json::to_string(&notification).unwrap();
            assert_eq!(ser, expect);
            let de = serde_json::from_str::<Notification>(expect).unwrap();
            assert_eq!(de, notification);
        }
    }

    #[test]
    fn call_serialization() {
        for (method_call, expect) in method_call_cases() {
            let call = Call::MethodCall(method_call);
            assert_eq!(serde_json::to_string(&call).unwrap(), expect);
            assert_eq!(serde_json::from_str::<Call>(expect).unwrap(), call);
        }

        for (notification, expect) in notification_cases() {
            let call = Call::Notification(notification);
            assert_eq!(serde_json::to_string(&call).unwrap(), expect);
            assert_eq!(serde_json::from_str::<Call>(expect).unwrap(), call);
        }
    }

    #[test]
    fn request_serialization() {
        for (method_call, expect) in method_call_cases() {
            let call_request = Request::Single(Call::MethodCall(method_call));
            assert_eq!(serde_json::to_string(&call_request).unwrap(), expect);
            assert_eq!(serde_json::from_str::<Request>(expect).unwrap(), call_request);
        }

        for (notification, expect) in notification_cases() {
            let notification_request = Request::Single(Call::Notification(notification));
            assert_eq!(serde_json::to_string(&notification_request).unwrap(), expect);
            assert_eq!(serde_json::from_str::<Request>(expect).unwrap(), notification_request);
        }

        let batch_request = Request::Batch(vec![
            Call::MethodCall(MethodCall::new("foo", None, 1.into())),
            Call::MethodCall(MethodCall::new("bar", None, 2.into())),
        ]);
        let batch_expect = r#"[{"jsonrpc":"2.0","method":"foo","id":1},{"jsonrpc":"2.0","method":"bar","id":2}]"#;
        assert_eq!(serde_json::to_string(&batch_request).unwrap(), batch_expect);
        assert_eq!(serde_json::from_str::<Request>(&batch_expect).unwrap(), batch_request);
    }

    #[test]
    fn invalid_request() {
        let cases = vec![
            // JSON-RPC 2.0 invalid request
            r#"{"jsonrpc":"2.0","method":"foo","params":[1,true],"id":1,"unknown":[]}"#,
            r#"{"jsonrpc":"2.0"`,"method":"foo","params":[1,true],"id":1.2}"#,
            r#"{"jsonrpc":"2.0","method":"foo","params":[1,true],"id":null,"unknown":[]}"#,
            r#"{"jsonrpc":"2.0","method":"foo","params":[1,true],"id":null}"#,
            r#"{"jsonrpc":"2.0","method":"foo","params":[1,true],"unknown":[]}"#,
            r#"{"jsonrpc":"2.0","method":"foo","unknown":[]}"#,
            r#"{"jsonrpc":"2.0","unknown":[]}"#,
        ];

        for case in cases {
            let request = serde_json::from_str::<Request>(case);
            assert!(request.is_err());
        }
    }

    #[test]
    fn valid_request() {
        let cases = vec![
            // JSON-RPC 2.0 valid request
            r#"{"jsonrpc":"2.0","method":"foo","params":[1,true],"id":1}"#,
            r#"{"jsonrpc":"2.0","method":"foo","params":[],"id":1}"#,
            r#"{"jsonrpc":"2.0","method":"foo","id":1}"#,
            r#"{"jsonrpc":"2.0","method":"foo","params":[1,true]}"#,
            r#"{"jsonrpc":"2.0","method":"foo","params":[]}"#,
            r#"{"jsonrpc":"2.0","method":"foo"}"#,
        ];

        for case in cases {
            let request = serde_json::from_str::<Request>(case);
            assert!(request.is_ok());
        }
    }
}
