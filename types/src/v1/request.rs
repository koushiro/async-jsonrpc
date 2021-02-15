use std::{fmt, marker::PhantomData};

use serde::{de, ser, Deserialize, Serialize};
use serde_json::Value;

use crate::id::Id;

/// Represents JSON-RPC 1.0 request parameters.
pub type Params = Vec<Value>;

/// Represents JSON-RPC 1.0 request which is a method call.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MethodCall {
    /// A String containing the name of the method to be invoked.
    ///
    /// Method names that begin with the word rpc followed by a period character (U+002E or ASCII 46)
    /// are reserved for rpc-internal methods and extensions and MUST NOT be used for anything else.
    pub method: String,
    /// A Structured value that holds the parameter values to be used
    /// during the invocation of the method. This member MAY be omitted.
    pub params: Params,
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
    /// Creates a JSON-RPC 1.0 request which is a method call.
    pub fn new<M: Into<String>>(method: M, params: Params, id: Id) -> Self {
        Self {
            method: method.into(),
            params,
            id,
        }
    }
}

/// Represents JSON-RPC 1.0 request which is a notification.
///
/// A Request object that is a Notification signifies the Client's lack of interest in the
/// corresponding Response object, and as such no Response object needs to be returned to the client.
/// As such, the Client would not be aware of any errors (like e.g. "Invalid params","Internal error").
///
/// The Server MUST NOT reply to a Notification, including those that are within a batch request.
///
/// For JSON-RPC 1.0 specification, notification id **MUST** be Null.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Notification {
    /// A String containing the name of the method to be invoked.
    ///
    /// Method names that begin with the word rpc followed by a period character (U+002E or ASCII 46)
    /// are reserved for rpc-internal methods and extensions and MUST NOT be used for anything else.
    pub method: String,
    /// A Structured value that holds the parameter values to be used
    /// during the invocation of the method.
    pub params: Params,
}

impl fmt::Display for Notification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let json = serde_json::to_string(self).expect("`Notification` is serializable");
        write!(f, "{}", json)
    }
}

impl ser::Serialize for Notification {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let mut state = ser::Serializer::serialize_struct(serializer, "Notification", 3)?;
        ser::SerializeStruct::serialize_field(&mut state, "method", &self.method)?;
        ser::SerializeStruct::serialize_field(&mut state, "params", &self.params)?;
        ser::SerializeStruct::serialize_field(&mut state, "id", &Option::<Id>::None)?;
        ser::SerializeStruct::end(state)
    }
}

impl<'de> de::Deserialize<'de> for Notification {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        use self::request_field::{Field, FIELDS};

        struct Visitor<'de> {
            marker: PhantomData<Notification>,
            lifetime: PhantomData<&'de ()>,
        }
        impl<'de> de::Visitor<'de> for Visitor<'de> {
            type Value = Notification;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("struct Notification")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let mut method = Option::<String>::None;
                let mut params = Option::<Params>::None;
                let mut id = Option::<Option<Id>>::None;

                while let Some(key) = de::MapAccess::next_key::<Field>(&mut map)? {
                    match key {
                        Field::Method => {
                            if method.is_some() {
                                return Err(de::Error::duplicate_field("method"));
                            }
                            method = Some(de::MapAccess::next_value::<String>(&mut map)?)
                        }
                        Field::Params => {
                            if params.is_some() {
                                return Err(de::Error::duplicate_field("params"));
                            }
                            params = Some(de::MapAccess::next_value::<Params>(&mut map)?)
                        }
                        Field::Id => {
                            if id.is_some() {
                                return Err(de::Error::duplicate_field("id"));
                            }
                            id = Some(de::MapAccess::next_value::<Option<Id>>(&mut map)?)
                        }
                    }
                }

                let method = method.ok_or_else(|| de::Error::missing_field("method"))?;
                let params = params.ok_or_else(|| de::Error::missing_field("params"))?;
                let id = id.ok_or_else(|| de::Error::missing_field("id"))?;
                if id.is_some() {
                    return Err(de::Error::custom(
                        "JSON-RPC 1.0 notification id MUST be Null",
                    ));
                }
                Ok(Notification { method, params })
            }
        }

        de::Deserializer::deserialize_struct(
            deserializer,
            "Notification",
            FIELDS,
            Visitor {
                marker: PhantomData::<Notification>,
                lifetime: PhantomData,
            },
        )
    }
}

impl Notification {
    /// Creates a JSON-RPC 1.0 request which is a notification.
    pub fn new<M: Into<String>>(method: M, params: Params) -> Self {
        Self {
            method: method.into(),
            params,
        }
    }
}

/// Represents single JSON-RPC 1.0 call.
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
    pub fn params(&self) -> &Params {
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

/// JSON-RPC 1.0 Request object (only for method call).
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

mod request_field {
    use super::*;

    pub const FIELDS: &[&str] = &["method", "params", "id"];
    pub enum Field {
        Method,
        Params,
        Id,
    }

    impl<'de> de::Deserialize<'de> for Field {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: de::Deserializer<'de>,
        {
            de::Deserializer::deserialize_identifier(deserializer, FieldVisitor)
        }
    }

    struct FieldVisitor;
    impl<'de> de::Visitor<'de> for FieldVisitor {
        type Value = Field;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("field identifier")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match v {
                "method" => Ok(Field::Method),
                "params" => Ok(Field::Params),
                "id" => Ok(Field::Id),
                _ => Err(de::Error::unknown_field(v, &FIELDS)),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn method_call_cases() -> Vec<(MethodCall, &'static str)> {
        vec![
            (
                // JSON-RPC 1.0 request method call
                MethodCall {
                    method: "foo".to_string(),
                    params: vec![Value::from(1), Value::Bool(true)],
                    id: Id::Num(1),
                },
                r#"{"method":"foo","params":[1,true],"id":1}"#,
            ),
            (
                // JSON-RPC 1.0 request method call without parameters
                MethodCall {
                    method: "foo".to_string(),
                    params: vec![],
                    id: Id::Num(1),
                },
                r#"{"method":"foo","params":[],"id":1}"#,
            ),
        ]
    }

    fn notification_cases() -> Vec<(Notification, &'static str)> {
        vec![
            (
                // JSON-RPC 1.0 request notification
                Notification {
                    method: "foo".to_string(),
                    params: vec![Value::from(1), Value::Bool(true)],
                },
                r#"{"method":"foo","params":[1,true],"id":null}"#,
            ),
            (
                // JSON-RPC 1.0 request notification without parameters
                Notification {
                    method: "foo".to_string(),
                    params: vec![],
                },
                r#"{"method":"foo","params":[],"id":null}"#,
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
            assert_eq!(
                serde_json::from_str::<Request>(expect).unwrap(),
                call_request
            );
        }

        for (notification, expect) in notification_cases() {
            let notification_request = Request::Single(Call::Notification(notification));
            assert_eq!(
                serde_json::to_string(&notification_request).unwrap(),
                expect
            );
            assert_eq!(
                serde_json::from_str::<Request>(expect).unwrap(),
                notification_request
            );
        }

        let batch_request = Request::Batch(vec![
            Call::MethodCall(MethodCall {
                method: "foo".into(),
                params: vec![],
                id: Id::Num(1),
            }),
            Call::MethodCall(MethodCall {
                method: "bar".into(),
                params: vec![],
                id: Id::Num(2),
            }),
        ]);
        let batch_expect =
            r#"[{"method":"foo","params":[],"id":1},{"method":"bar","params":[],"id":2}]"#;
        assert_eq!(serde_json::to_string(&batch_request).unwrap(), batch_expect);
        assert_eq!(
            serde_json::from_str::<Request>(&batch_expect).unwrap(),
            batch_request
        );
    }

    #[test]
    fn invalid_request() {
        let cases = vec![
            // JSON-RPC 1.0 invalid request
            r#"{"method":"foo","params":[1,true],"id":1,"unknown":[]}"#,
            r#"{"method":"foo","params":[1,true],"id":1.2}"#,
            r#"{"method":"foo","params":[1,true],"id":null,"unknown":[]}"#,
            r#"{"method":"foo","params":[1,true],"unknown":[]}"#,
            r#"{"method":"foo","params":[1,true]}"#,
            r#"{"method":"foo","unknown":[]}"#,
            r#"{"method":1,"unknown":[]}"#,
            r#"{"unknown":[]}"#,
        ];

        for case in cases {
            let request = serde_json::from_str::<Request>(case);
            assert!(request.is_err());
        }
    }

    #[test]
    fn valid_request() {
        let cases = vec![
            // JSON-RPC 1.0 valid request
            r#"{"method":"foo","params":[1,true],"id":1}"#,
            r#"{"method":"foo","params":[],"id":1}"#,
            r#"{"method":"foo","params":[1,true],"id":null}"#,
            r#"{"method":"foo","params":[],"id":null}"#,
        ];

        for case in cases {
            let request = serde_json::from_str::<Request>(case);
            assert!(request.is_ok());
        }
    }
}
