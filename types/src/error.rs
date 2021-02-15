use std::{error, fmt};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

/// JSON-RPC Error Code.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ErrorCode {
    /// Invalid JSON was received by the server.
    /// An error occurred on the server while parsing the JSON text.
    ParseError,
    /// The JSON sent is not a valid Request object.
    InvalidRequest,
    /// The method does not exist / is not available.
    MethodNotFound,
    /// Invalid method parameter(s).
    InvalidParams,
    /// Internal JSON-RPC error.
    InternalError,
    /// Reserved for implementation-defined server-errors.
    ServerError(i64),
}

impl From<i64> for ErrorCode {
    fn from(code: i64) -> Self {
        match code {
            -32700 => ErrorCode::ParseError,
            -32600 => ErrorCode::InvalidRequest,
            -32601 => ErrorCode::MethodNotFound,
            -32602 => ErrorCode::InvalidParams,
            -32603 => ErrorCode::InternalError,
            code => ErrorCode::ServerError(code),
        }
    }
}

impl Serialize for ErrorCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i64(self.code())
    }
}

impl<'de> Deserialize<'de> for ErrorCode {
    fn deserialize<D>(deserializer: D) -> Result<ErrorCode, D::Error>
    where
        D: Deserializer<'de>,
    {
        let code: i64 = Deserialize::deserialize(deserializer)?;
        Ok(ErrorCode::from(code))
    }
}

impl ErrorCode {
    /// Returns integer code value.
    pub fn code(&self) -> i64 {
        match self {
            ErrorCode::ParseError => -32700,
            ErrorCode::InvalidRequest => -32600,
            ErrorCode::MethodNotFound => -32601,
            ErrorCode::InvalidParams => -32602,
            ErrorCode::InternalError => -32603,
            ErrorCode::ServerError(code) => *code,
        }
    }

    /// Returns human-readable description.
    pub fn description(&self) -> String {
        let desc = match self {
            ErrorCode::ParseError => "Parse error",
            ErrorCode::InvalidRequest => "Invalid request",
            ErrorCode::MethodNotFound => "Method not found",
            ErrorCode::InvalidParams => "Invalid params",
            ErrorCode::InternalError => "Internal error",
            ErrorCode::ServerError(_) => "Server error",
        };
        desc.to_string()
    }
}

/// JSON-RPC Error Object.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Error {
    /// A Number that indicates the error type that occurred.
    /// This MUST be an integer.
    pub code: ErrorCode,
    /// A String providing a short description of the error.
    /// The message SHOULD be limited to a concise single sentence.
    pub message: String,
    /// A Primitive or Structured value that contains additional information about the error.
    /// This may be omitted.
    /// The value of this member is defined by the Server (e.g. detailed error information, nested errors etc.).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.code.description(), self.message)
    }
}

impl error::Error for Error {}

impl Error {
    /// Wraps given `ErrorCode`.
    pub fn new(code: ErrorCode) -> Self {
        Error {
            message: code.description(),
            code,
            data: None,
        }
    }

    /// Creates a new `ParseError` error.
    pub fn parse_error() -> Self {
        Self::new(ErrorCode::ParseError)
    }

    /// Creates a new `InvalidRequest` error.
    pub fn invalid_request() -> Self {
        Self::new(ErrorCode::InvalidRequest)
    }

    /// Creates a new `MethodNotFound` error.
    pub fn method_not_found() -> Self {
        Self::new(ErrorCode::MethodNotFound)
    }

    /// Creates a new `InvalidParams` error with given message.
    pub fn invalid_params<M>(message: M) -> Self
    where
        M: fmt::Display,
    {
        Error {
            code: ErrorCode::InvalidParams,
            message: format!("Invalid parameters: {}", message),
            data: None,
        }
    }

    /// Creates a new `InvalidParams` error with given message and details.
    pub fn invalid_params_with_details<M, D>(message: M, details: D) -> Self
    where
        M: fmt::Display,
        D: fmt::Display,
    {
        Error {
            code: ErrorCode::InvalidParams,
            message: format!("Invalid parameters: {}", message),
            data: Some(Value::String(details.to_string())),
        }
    }

    /// Creates a new `InternalError` error.
    pub fn internal_error() -> Self {
        Self::new(ErrorCode::InternalError)
    }

    /// Creates a new `InvalidRequest` error with invalid version description.
    pub fn invalid_version() -> Self {
        Error {
            code: ErrorCode::InvalidRequest,
            message: "Unsupported JSON-RPC protocol version".to_owned(),
            data: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_serialization() {
        assert_eq!(
            serde_json::to_string(&Error::parse_error()).unwrap(),
            r#"{"code":-32700,"message":"Parse error"}"#
        );
        assert_eq!(
            serde_json::to_string(&Error::invalid_request()).unwrap(),
            r#"{"code":-32600,"message":"Invalid request"}"#
        );
        assert_eq!(
            serde_json::to_string(&Error::method_not_found()).unwrap(),
            r#"{"code":-32601,"message":"Method not found"}"#
        );
        assert_eq!(
            serde_json::to_string(&Error::invalid_params("unexpected params")).unwrap(),
            r#"{"code":-32602,"message":"Invalid parameters: unexpected params"}"#
        );
        assert_eq!(
            serde_json::to_string(&Error::invalid_params_with_details(
                "unexpected params",
                "details"
            ))
            .unwrap(),
            r#"{"code":-32602,"message":"Invalid parameters: unexpected params","data":"details"}"#
        );
        assert_eq!(
            serde_json::to_string(&Error::internal_error()).unwrap(),
            r#"{"code":-32603,"message":"Internal error"}"#
        );
        assert_eq!(
            serde_json::to_string(&Error::invalid_version()).unwrap(),
            r#"{"code":-32600,"message":"Unsupported JSON-RPC protocol version"}"#
        );
    }
}
