#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};
use core::{fmt, marker::PhantomData};

use serde::{de, ser};

use crate::v1::{Id, Params, ParamsRef};

/// Represents JSON-RPC 1.0 batch notification.
pub type BatchNotificationRef<'a> = Vec<NotificationRef<'a>>;

/// Represents JSON-RPC 1.0 request which is a notification.
///
/// A Request object that is a Notification signifies the Client's lack of interest in the
/// corresponding Response object, and as such no Response object needs to be returned to the client.
/// As such, the Client would not be aware of any errors (like e.g. "Invalid params","Internal error").
///
/// The Server MUST NOT reply to a Notification, including those that are within a batch request.
///
/// For JSON-RPC 1.0 specification, notification id **MUST** be Null.
#[derive(Clone, Debug, PartialEq)]
pub struct NotificationRef<'a> {
    /// A String containing the name of the method to be invoked.
    ///
    /// Method names that begin with the word rpc followed by a period character (U+002E or ASCII 46)
    /// are reserved for rpc-internal methods and extensions and MUST NOT be used for anything else.
    pub method: &'a str,
    /// A Structured value that holds the parameter values to be used
    /// during the invocation of the method.
    pub params: ParamsRef<'a>,
}

impl<'a> fmt::Display for NotificationRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let json = serde_json::to_string(self).expect("`Notification` is serializable");
        write!(f, "{}", json)
    }
}

impl<'a> PartialEq<Notification> for NotificationRef<'a> {
    fn eq(&self, other: &Notification) -> bool {
        self.method.eq(&other.method) && self.params.eq(&other.params)
    }
}

impl<'a> ser::Serialize for NotificationRef<'a> {
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

impl<'a> NotificationRef<'a> {
    /// Creates a JSON-RPC 1.0 request which is a notification.
    pub fn new(method: &'a str, params: ParamsRef<'a>) -> Self {
        Self { method, params }
    }

    /// Converts the reference into the owned type.
    pub fn to_owned(&self) -> Notification {
        Notification {
            method: self.method.into(),
            params: self.params.to_vec(),
        }
    }
}

// ################################################################################################

/// Represents JSON-RPC 1.0 batch notification.
pub type BatchNotification = Vec<Notification>;

/// Represents JSON-RPC 1.0 request which is a notification.
///
/// A Request object that is a Notification signifies the Client's lack of interest in the
/// corresponding Response object, and as such no Response object needs to be returned to the client.
/// As such, the Client would not be aware of any errors (like e.g. "Invalid params","Internal error").
///
/// The Server MUST NOT reply to a Notification, including those that are within a batch request.
///
/// For JSON-RPC 1.0 specification, notification id **MUST** be Null.
#[derive(Clone, Debug, PartialEq)]
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

impl<'a> PartialEq<NotificationRef<'a>> for Notification {
    fn eq(&self, other: &NotificationRef<'a>) -> bool {
        self.method.eq(other.method) && self.params.eq(other.params)
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
                    return Err(de::Error::custom("JSON-RPC 1.0 notification id MUST be Null"));
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
    pub fn new<M: Into<String>, P: Into<Params>>(method: M, params: P) -> Self {
        Self {
            method: method.into(),
            params: params.into(),
        }
    }

    /// Borrows from an owned value.
    pub fn as_ref(&self) -> NotificationRef<'_> {
        NotificationRef {
            method: &self.method,
            params: &self.params,
        }
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
    use serde_json::Value;

    use super::*;

    fn notification_cases() -> Vec<(Notification, &'static str)> {
        vec![
            (
                // JSON-RPC 1.0 request notification
                Notification::new("foo", vec![Value::from(1), Value::Bool(true)]),
                r#"{"method":"foo","params":[1,true],"id":null}"#,
            ),
            (
                // JSON-RPC 1.0 request notification without parameters
                Notification::new("foo", vec![]),
                r#"{"method":"foo","params":[],"id":null}"#,
            ),
        ]
    }

    #[test]
    fn notification_serialization() {
        for (notification, expect) in notification_cases() {
            assert_eq!(serde_json::to_string(&notification.as_ref()).unwrap(), expect);
            assert_eq!(serde_json::to_string(&notification).unwrap(), expect);

            assert_eq!(serde_json::from_str::<Notification>(expect).unwrap(), notification);
        }

        // JSON-RPC 1.0 valid notification
        let valid_cases = vec![
            r#"{"method":"foo","params":[1,true],"id":null}"#,
            r#"{"method":"foo","params":[],"id":null}"#,
        ];
        for case in valid_cases {
            assert!(serde_json::from_str::<Notification>(case).is_ok());
        }

        // JSON-RPC 1.0 invalid notification
        let invalid_cases = vec![
            r#"{"method":"foo","params":[1,true],"id":1,"unknown":[]}"#,
            r#"{"method":"foo","params":[1,true],"id":1.2}"#,
            r#"{"method":"foo","params":[1,true],"id":null,"unknown":[]}"#,
            r#"{"method":"foo","params":[1,true],"unknown":[]}"#,
            r#"{"method":"foo","params":[1,true]}"#,
            r#"{"method":"foo","unknown":[]}"#,
            r#"{"method":1,"unknown":[]}"#,
            r#"{"unknown":[]}"#,
        ];
        for case in invalid_cases {
            assert!(serde_json::from_str::<Notification>(case).is_err());
        }
    }

    #[test]
    fn batch_notification_serialization() {
        let batch_notification = vec![Notification::new("foo", vec![]), Notification::new("bar", vec![])];
        let batch_notification_ref = batch_notification.iter().map(|n| n.as_ref()).collect::<Vec<_>>();
        let batch_expect = r#"[{"method":"foo","params":[],"id":null},{"method":"bar","params":[],"id":null}]"#;

        assert_eq!(serde_json::to_string(&batch_notification).unwrap(), batch_expect);
        assert_eq!(serde_json::to_string(&batch_notification_ref).unwrap(), batch_expect);

        assert_eq!(
            serde_json::from_str::<BatchNotification>(&batch_expect).unwrap(),
            batch_notification,
        );
    }
}
