#[cfg(not(feature = "std"))]
use alloc::{string::String, vec, vec::Vec};
use core::fmt;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::v1::{Id, Params, ParamsRef};

/// JSON-RPC 2.0 Request Object.
#[derive(Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum RequestRefObj<'a> {
    /// Single request call
    Single(RequestRef<'a>),
    /// Batch of request calls
    Batch(BatchRequestRef<'a>),
}

impl<'a> fmt::Display for RequestRefObj<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let json = serde_json::to_string(self).expect("`RequestRefObj` is serializable");
        write!(f, "{}", json)
    }
}

/// Represents JSON-RPC 1.0 batch request call.
pub type BatchRequestRef<'a> = Vec<RequestRef<'a>>;

/// Represents JSON-RPC 1.0 request call.
#[derive(Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RequestRef<'a> {
    /// A String containing the name of the method to be invoked.
    ///
    /// Method names that begin with the word rpc followed by a period character (U+002E or ASCII 46)
    /// are reserved for rpc-internal methods and extensions and MUST NOT be used for anything else.
    pub method: &'a str,
    /// A Structured value that holds the parameter values to be used
    /// during the invocation of the method. This member MAY be omitted.
    pub params: ParamsRef<'a>,
    /// An identifier established by the Client.
    /// If it is not included it is assumed to be a notification.
    pub id: Id,
}

impl<'a> fmt::Display for RequestRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let json = serde_json::to_string(self).expect("`RequestRef` is serializable");
        write!(f, "{}", json)
    }
}

impl<'a> PartialEq<Request> for RequestRef<'a> {
    fn eq(&self, other: &Request) -> bool {
        self.method.eq(&other.method) && self.params.eq(&other.params) && self.id.eq(&other.id)
    }
}

impl<'a> RequestRef<'a> {
    /// Creates a JSON-RPC 1.0 request which is a call.
    pub fn new(method: &'a str, params: ParamsRef<'a>, id: Id) -> Self {
        Self { method, params, id }
    }

    /// Converts the reference into the owned type.
    pub fn to_owned(&self) -> Request {
        Request {
            method: self.method.into(),
            params: self.params.to_vec(),
            id: self.id.clone(),
        }
    }
}

// ################################################################################################

/// JSON-RPC 2.0 Request Object.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum RequestObj {
    /// Single request call
    Single(Request),
    /// Batch of request calls
    Batch(BatchRequest),
}

impl fmt::Display for RequestObj {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let json = serde_json::to_string(self).expect("`RequestObj` is serializable");
        write!(f, "{}", json)
    }
}

/// Represents JSON-RPC 1.0 batch request call.
pub type BatchRequest = Vec<Request>;

/// Represents JSON-RPC 1.0 request call.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
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

impl fmt::Display for Request {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let json = serde_json::to_string(self).expect("`Request` is serializable");
        write!(f, "{}", json)
    }
}

impl<'a> PartialEq<RequestRef<'a>> for Request {
    fn eq(&self, other: &RequestRef<'a>) -> bool {
        self.method.eq(other.method) && self.params.eq(other.params) && self.id.eq(&other.id)
    }
}

impl Request {
    /// Creates a JSON-RPC 1.0 request which is a call.
    pub fn new<M: Into<String>, P: Into<Params>>(method: M, params: P, id: Id) -> Self {
        Self {
            method: method.into(),
            params: params.into(),
            id,
        }
    }

    /// Borrows from an owned value.
    pub fn as_ref(&self) -> RequestRef<'_> {
        RequestRef {
            method: &self.method,
            params: &self.params,
            id: self.id.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request_cases() -> Vec<(Request, &'static str)> {
        vec![
            (
                // JSON-RPC 1.0 request
                Request {
                    method: "foo".into(),
                    params: vec![Value::from(1), Value::Bool(true)].into(),
                    id: Id::Num(1),
                },
                r#"{"method":"foo","params":[1,true],"id":1}"#,
            ),
            (
                // JSON-RPC 1.0 request without parameters
                Request {
                    method: "foo".into(),
                    params: vec![].into(),
                    id: Id::Num(1),
                },
                r#"{"method":"foo","params":[],"id":1}"#,
            ),
        ]
    }

    #[test]
    fn request_serialization() {
        for (request, expect) in request_cases() {
            let ser = serde_json::to_string(&request).unwrap();
            assert_eq!(ser, expect);
            let de = serde_json::from_str::<Request>(expect).unwrap();
            assert_eq!(de, request);
        }

        // JSON-RPC 1.0 valid request
        let valid_cases = vec![
            r#"{"method":"foo","params":[1,true],"id":1}"#,
            r#"{"method":"foo","params":[],"id":1}"#,
        ];
        for case in valid_cases {
            let request = serde_json::from_str::<Request>(case);
            assert!(request.is_ok());
        }

        // JSON-RPC 1.0 invalid request
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
            let request = serde_json::from_str::<Request>(case);
            assert!(request.is_err());
        }
    }

    #[test]
    fn batch_request_serialization() {
        let batch_request = vec![
            Request {
                method: "foo".into(),
                params: vec![].into(),
                id: Id::Num(1),
            },
            Request {
                method: "bar".into(),
                params: vec![].into(),
                id: Id::Num(2),
            },
        ];
        let batch_expect = r#"[{"method":"foo","params":[],"id":1},{"method":"bar","params":[],"id":2}]"#;
        assert_eq!(serde_json::to_string(&batch_request).unwrap(), batch_expect);
        assert_eq!(
            serde_json::from_str::<BatchRequest>(&batch_expect).unwrap(),
            batch_request
        );
    }
}
