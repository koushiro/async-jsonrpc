#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
use core::fmt;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

use crate::v2::{Error, ErrorCode, Id, Version};

/// JSON-RPC 2.0 Response Object.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum ResponseObj<T = Value> {
    /// Single response
    Single(Response<T>),
    /// Response to batch request (batch of responses)
    Batch(Vec<Response<T>>),
}

impl<T: Serialize> fmt::Display for ResponseObj<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let json = serde_json::to_string(self).expect("`Response` is serializable");
        write!(f, "{}", json)
    }
}

impl<T> From<Success<T>> for ResponseObj<T> {
    fn from(success: Success<T>) -> Self {
        Self::Single(Response::<T>::Success(success))
    }
}

impl<T> From<Failure> for ResponseObj<T> {
    fn from(failure: Failure) -> Self {
        Self::Single(Response::<T>::Failure(failure))
    }
}

impl<T> From<BatchResponse<T>> for ResponseObj<T> {
    fn from(batch: BatchResponse<T>) -> Self {
        Self::Batch(batch)
    }
}

/// Represents JSON-RPC 2.0 batch response.
pub type BatchResponse<T = Value> = Vec<Response<T>>;

/// Represents JSON-RPC 2.0 success / failure response.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum Response<T = Value> {
    /// Success response
    Success(Success<T>),
    /// Failure response
    Failure(Failure),
}

impl<T: Serialize> fmt::Display for Response<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let json = serde_json::to_string(self).expect("`Response` is serializable");
        write!(f, "{}", json)
    }
}

impl<T> From<Success<T>> for Response<T> {
    fn from(success: Success<T>) -> Self {
        Self::Success(success)
    }
}

impl<T> From<Failure> for Response<T> {
    fn from(failure: Failure) -> Self {
        Self::Failure(failure)
    }
}

impl<T: Serialize + DeserializeOwned> Response<T> {
    /// Creates a JSON-RPC 2.0 success response.
    pub fn success(result: T, id: Id) -> Self {
        Self::Success(Success::new(result, id))
    }

    /// Creates a JSON-RPC 2.0 failure response.
    pub fn failure(error: Error, id: Option<Id>) -> Self {
        Self::Failure(Failure::new(error, id))
    }

    /// Gets the JSON-RPC protocol version.
    pub fn version(&self) -> Version {
        match self {
            Self::Success(s) => s.jsonrpc,
            Self::Failure(f) => f.jsonrpc,
        }
    }

    /// Gets the correlation id.
    pub fn id(&self) -> Option<Id> {
        match self {
            Self::Success(s) => Some(s.id.clone()),
            Self::Failure(f) => f.id.clone(),
        }
    }
}

impl<T: Serialize + DeserializeOwned> From<Response<T>> for Result<T, Error> {
    // Convert into a result.
    // Will be `Ok` if it is a `SuccessResponse` and `Err` if `FailureResponse`.
    fn from(response: Response<T>) -> Result<T, Error> {
        match response {
            Response::Success(s) => Ok(s.result),
            Response::Failure(f) => Err(f.error),
        }
    }
}

/// Represents JSON-RPC 2.0 success response.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Success<T = Value> {
    /// A String specifying the version of the JSON-RPC protocol.
    pub jsonrpc: Version,
    /// Successful execution result.
    pub result: T,
    /// Correlation id.
    ///
    /// It **MUST** be the same as the value of the id member in the Request Object.
    pub id: Id,
}

impl<T: Serialize> fmt::Display for Success<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let json = serde_json::to_string(self).expect("`Success` is serializable");
        write!(f, "{}", json)
    }
}

impl<T: Serialize + DeserializeOwned> Success<T> {
    /// Creates a JSON-RPC 2.0 success response.
    pub fn new(result: T, id: Id) -> Self {
        Self {
            jsonrpc: Version::V2_0,
            result,
            id,
        }
    }
}

/// Represents JSON-RPC 2.0 failure response.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Failure {
    /// A String specifying the version of the JSON-RPC protocol.
    pub jsonrpc: Version,
    /// Failed execution error.
    pub error: Error,
    /// Correlation id.
    ///
    /// It **MUST** be the same as the value of the id member in the Request Object.
    ///
    /// If there was an error in detecting the id in the Request object (e.g. Parse error/Invalid Request),
    /// it **MUST** be Null.
    pub id: Option<Id>,
}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let json = serde_json::to_string(self).expect("`Failure` is serializable");
        write!(f, "{}", json)
    }
}

impl Failure {
    /// Creates a JSON-RPC 2.0 failure response.
    pub fn new(error: Error, id: Option<Id>) -> Self {
        Self {
            jsonrpc: Version::V2_0,
            error,
            id,
        }
    }

    /// Creates a JSON-RPC 2.0 failure response, indicating that the server has an error in parsing the JSON text.
    pub fn parse_error(id: Option<Id>) -> Self {
        Self::new(Error::parse_error(), id)
    }

    /// Creates a JSON-RPC 2.0 failure response, indicating malformed request.
    pub fn invalid_request(id: Option<Id>) -> Self {
        Self::new(Error::invalid_request(), id)
    }

    /// Creates a JSON-RPC 2.0 failure response, indicating that the request's method is not found.
    pub fn method_not_found(id: Id) -> Self {
        Self::new(Error::method_not_found(), Some(id))
    }

    /// Creates a JSON-RPC 2.0 failure response, indicating that the request's parameters is invalid.
    pub fn invalid_params(id: Id, msg: impl fmt::Display) -> Self {
        Self::new(Error::invalid_params(msg), Some(id))
    }

    /// Creates a JSON-RPC 2.0 failure response, indicating that the internal JSON-RPC error.
    pub fn internal_error(id: Id) -> Self {
        Self::new(Error::internal_error(), Some(id))
    }

    /// Creates a JSON-RPC 2.0 failure response, indicating that implementation-defined server error.
    pub fn server_error(id: Id, error: i64) -> Self {
        Self::new(Error::new(ErrorCode::ServerError(error)), Some(id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // JSON-RPC 2.0 success response
    fn success_response_cases() -> Vec<(Success, &'static str)> {
        vec![(
            Success::new(Value::Bool(true), Id::Num(1)),
            r#"{"jsonrpc":"2.0","result":true,"id":1}"#,
        )]
    }

    // JSON-RPC 2.0 failure response
    fn failure_response_cases() -> Vec<(Failure, &'static str)> {
        vec![
            (
                Failure::parse_error(Some(Id::Num(1))),
                r#"{"jsonrpc":"2.0","error":{"code":-32700,"message":"Parse error"},"id":1}"#,
            ),
            (
                Failure::parse_error(None),
                r#"{"jsonrpc":"2.0","error":{"code":-32700,"message":"Parse error"},"id":null}"#,
            ),
        ]
    }

    #[test]
    fn response_serialization() {
        for (success_response, expect) in success_response_cases() {
            assert_eq!(serde_json::to_string(&success_response).unwrap(), expect);
            assert_eq!(serde_json::from_str::<Success>(expect).unwrap(), success_response);

            let response = Response::Success(success_response);
            assert_eq!(serde_json::to_string(&response).unwrap(), expect);
            assert_eq!(serde_json::from_str::<Response>(expect).unwrap(), response);

            let response = ResponseObj::Single(response.clone());
            assert_eq!(serde_json::to_string(&response).unwrap(), expect);
            assert_eq!(serde_json::from_str::<ResponseObj>(expect).unwrap(), response);
        }

        for (failure_response, expect) in failure_response_cases() {
            assert_eq!(serde_json::to_string(&failure_response).unwrap(), expect);
            assert_eq!(serde_json::from_str::<Failure>(expect).unwrap(), failure_response);

            let response = Response::Failure(failure_response);
            assert_eq!(serde_json::to_string(&response).unwrap(), expect);
            assert_eq!(serde_json::from_str::<Response>(expect).unwrap(), response);

            let response = ResponseObj::Single(response.clone());
            assert_eq!(serde_json::to_string(&response).unwrap(), expect);
            assert_eq!(serde_json::from_str::<ResponseObj>(expect).unwrap(), response);
        }

        // JSON-RPC 2.0 valid response
        let valid_cases = vec![
            r#"{"jsonrpc":"2.0","result":true,"id":1}"#,
            r#"{"jsonrpc":"2.0","error":{"code": -32700,"message": "Parse error"},"id":1}"#,
            r#"{"jsonrpc":"2.0","error":{"code": -32700,"message": "Parse error"},"id":null}"#,
        ];
        for case in valid_cases {
            assert!(serde_json::from_str::<Response>(case).is_ok());
            assert!(serde_json::from_str::<ResponseObj>(case).is_ok());
        }

        // JSON-RPC 2.0 invalid response
        let invalid_cases = vec![
            r#"{"jsonrpc":"2.0","result":true,"id":1,"unknown":[]}"#,
            r#"{"jsonrpc":"2.0","error":{"code": -32700,"message": "Parse error"},"id":1,"unknown":[]}"#,
            r#"{"jsonrpc":"2.0","result":true,"error":{"code": -32700,"message": "Parse error"},"id":1}"#,
            r#"{"jsonrpc":"2.0","id":1}"#,
            r#"{"jsonrpc":"2.0","unknown":[]}"#,
        ];
        for case in invalid_cases {
            assert!(serde_json::from_str::<Response>(case).is_err());
            assert!(serde_json::from_str::<ResponseObj>(case).is_err());
        }
    }

    #[test]
    fn batch_response_serialization() {
        for ((success_resp, success_expect), (failure_resp, failure_expect)) in
            success_response_cases().into_iter().zip(failure_response_cases())
        {
            let response = vec![Response::Success(success_resp), Response::Failure(failure_resp)];
            let response_obj = ResponseObj::Batch(response.clone());
            let expect = format!("[{},{}]", success_expect, failure_expect);

            assert_eq!(serde_json::to_string(&response).unwrap(), expect);
            assert_eq!(serde_json::to_string(&response_obj).unwrap(), expect);

            assert_eq!(serde_json::from_str::<BatchResponse>(&expect).unwrap(), response);
            assert_eq!(serde_json::from_str::<ResponseObj>(&expect).unwrap(), response_obj);
        }
    }
}
