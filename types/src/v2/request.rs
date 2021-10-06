#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};
use core::fmt;

use serde::{Deserialize, Serialize};

use crate::v2::{Id, Params, ParamsRef, Version};

/// JSON-RPC 2.0 Request Object.
#[derive(Clone, Debug, PartialEq, Serialize)]
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

/// Represents JSON-RPC 2.0 batch request call.
pub type BatchRequestRef<'a> = Vec<RequestRef<'a>>;

/// Represents JSON-RPC 2.0 request call.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RequestRef<'a> {
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
        self.method.eq(&other.method)
            && self.params.eq(&other.params.as_ref().map(|params| params.as_ref()))
            && self.id.eq(&other.id)
    }
}

impl<'a> RequestRef<'a> {
    /// Creates a JSON-RPC 2.0 request call.
    pub fn new(method: &'a str, params: Option<ParamsRef<'a>>, id: Id) -> Self {
        Self {
            jsonrpc: Version::V2_0,
            method,
            params,
            id,
        }
    }

    /// Converts the reference into the owned type.
    pub fn to_owned(&self) -> Request {
        Request {
            jsonrpc: self.jsonrpc,
            method: self.method.into(),
            params: self.params.as_ref().map(|params| params.to_owned()),
            id: self.id.clone(),
        }
    }
}

// ################################################################################################

/// JSON-RPC 2.0 Request Object.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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

/// Represents JSON-RPC 2.0 batch request call.
pub type BatchRequest = Vec<Request>;

/// Represents JSON-RPC 2.0 request call.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
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

impl fmt::Display for Request {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let json = serde_json::to_string(self).expect("`MethodCall` is serializable");
        write!(f, "{}", json)
    }
}

impl<'a> PartialEq<RequestRef<'a>> for Request {
    fn eq(&self, other: &RequestRef<'a>) -> bool {
        self.method.eq(other.method)
            && self.params.as_ref().map(|params| params.as_ref()).eq(&other.params)
            && self.id.eq(&other.id)
    }
}

impl Request {
    /// Creates a JSON-RPC 2.0 request call.
    pub fn new<M: Into<String>>(method: M, params: Option<Params>, id: Id) -> Self {
        Self {
            jsonrpc: Version::V2_0,
            method: method.into(),
            params,
            id,
        }
    }

    /// Borrows from an owned value.
    pub fn as_ref(&self) -> RequestRef<'_> {
        RequestRef {
            jsonrpc: self.jsonrpc,
            method: &self.method,
            params: self.params.as_ref().map(|params| params.as_ref()),
            id: self.id.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    fn request_cases() -> Vec<(Request, &'static str)> {
        vec![
            (
                // JSON-RPC 2.0 request call
                Request::new(
                    "foo",
                    Some(Params::Array(vec![Value::from(1), Value::Bool(true)])),
                    Id::Num(1),
                ),
                r#"{"jsonrpc":"2.0","method":"foo","params":[1,true],"id":1}"#,
            ),
            (
                // JSON-RPC 2.0 request method call with an empty array parameters
                Request::new("foo", Some(Params::Array(vec![])), Id::Num(1)),
                r#"{"jsonrpc":"2.0","method":"foo","params":[],"id":1}"#,
            ),
            (
                // JSON-RPC 2.0 request method call without parameters
                Request::new("foo", None, Id::Num(1)),
                r#"{"jsonrpc":"2.0","method":"foo","id":1}"#,
            ),
        ]
    }

    #[test]
    fn request_serialization() {
        for (request, expect) in request_cases() {
            let request_obj = RequestObj::Single(request.clone());
            let request_ref = RequestRefObj::Single(request.as_ref());

            assert_eq!(serde_json::to_string(&request).unwrap(), expect);
            assert_eq!(serde_json::to_string(&request_obj).unwrap(), expect);
            assert_eq!(serde_json::to_string(&request_ref).unwrap(), expect);

            assert_eq!(serde_json::from_str::<Request>(expect).unwrap(), request);
            assert_eq!(serde_json::from_str::<RequestObj>(expect).unwrap(), request_obj);
        }

        // JSON-RPC 2.0 valid request
        let valid_cases = vec![
            r#"{"jsonrpc":"2.0","method":"foo","params":[1,true],"id":1}"#,
            r#"{"jsonrpc":"2.0","method":"foo","params":[],"id":1}"#,
            r#"{"jsonrpc":"2.0","method":"foo","id":1}"#,
        ];
        for case in valid_cases {
            assert!(serde_json::from_str::<Request>(case).is_ok());
            assert!(serde_json::from_str::<RequestObj>(case).is_ok());
        }

        // JSON-RPC 2.0 invalid request
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
            assert!(serde_json::from_str::<Request>(case).is_err());
            assert!(serde_json::from_str::<RequestObj>(case).is_err());
        }
    }

    #[test]
    fn batch_request_serialization() {
        let batch_request = vec![Request::new("foo", None, 1.into()), Request::new("bar", None, 2.into())];
        let batch_request_obj = RequestObj::Batch(batch_request.clone());
        let batch_request_ref = RequestRefObj::Batch(batch_request.iter().map(|req| req.as_ref()).collect::<Vec<_>>());
        let batch_expect = r#"[{"jsonrpc":"2.0","method":"foo","id":1},{"jsonrpc":"2.0","method":"bar","id":2}]"#;

        assert_eq!(serde_json::to_string(&batch_request).unwrap(), batch_expect);
        assert_eq!(serde_json::to_string(&batch_request_obj).unwrap(), batch_expect);
        assert_eq!(serde_json::to_string(&batch_request_ref).unwrap(), batch_expect);

        assert_eq!(
            serde_json::from_str::<BatchRequest>(&batch_expect).unwrap(),
            batch_request
        );
        assert_eq!(
            serde_json::from_str::<RequestObj>(&batch_expect).unwrap(),
            batch_request_obj
        );
    }
}
