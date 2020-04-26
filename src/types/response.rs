use serde::{Deserialize, Serialize};

use crate::types::{Error, ErrorCode, RequestId, Value, Version};

/// Successful response
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuccessResponse {
    /// Protocol version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jsonrpc: Option<Version>,
    /// Correlation id
    pub id: RequestId,
    /// Result
    pub result: Value,
    /// Error
    pub error: Option<Error>,
}

/// Unsuccessful response
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FailureResponse {
    /// Protocol Version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jsonrpc: Option<Version>,
    /// Correlation id
    pub id: RequestId,
    /// Result
    pub result: Option<Value>,
    /// Error
    pub error: Error,
}

/// Represents output of response - failure or success
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum ResponseOutput {
    /// Success
    Success(SuccessResponse),
    /// Failure
    Failure(FailureResponse),
}

impl ResponseOutput {
    /// Creates new output given  `Version`, `Id` and `Result`.
    pub fn from(jsonrpc: Option<Version>, id: RequestId, result: Result<Value, Error>) -> Self {
        match result {
            Ok(result) => ResponseOutput::Success(SuccessResponse {
                jsonrpc,
                id,
                result,
                error: None,
            }),
            Err(error) => ResponseOutput::Failure(FailureResponse {
                jsonrpc,
                id,
                result: None,
                error,
            }),
        }
    }

    /// Creates new failure output indicating malformed request.
    pub fn invalid_request(jsonrpc: Option<Version>, id: RequestId) -> Self {
        ResponseOutput::Failure(FailureResponse {
            jsonrpc,
            id,
            result: None,
            error: Error::new(ErrorCode::InvalidRequest),
        })
    }

    /// Get the JSON-RPC protocol version.
    pub fn version(&self) -> Option<Version> {
        match self {
            ResponseOutput::Success(s) => s.jsonrpc,
            ResponseOutput::Failure(f) => f.jsonrpc,
        }
    }

    /// Get the correlation id.
    pub fn id(&self) -> RequestId {
        match self {
            ResponseOutput::Success(s) => s.id,
            ResponseOutput::Failure(f) => f.id,
        }
    }
}

impl From<ResponseOutput> for Result<Value, Error> {
    /// Convert into a result. Will be `Ok` if it is a `Success` and `Err` if `Failure`.
    fn from(output: ResponseOutput) -> Result<Value, Error> {
        match output {
            ResponseOutput::Success(s) => Ok(s.result),
            ResponseOutput::Failure(f) => Err(f.error),
        }
    }
}

/// Synchronous response
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum Response {
    /// Single response
    Single(ResponseOutput),
    /// Response to batch request (batch of responses)
    Batch(Vec<ResponseOutput>),
}

impl From<SuccessResponse> for Response {
    fn from(success: SuccessResponse) -> Self {
        Response::Single(ResponseOutput::Success(success))
    }
}

impl From<FailureResponse> for Response {
    fn from(failure: FailureResponse) -> Self {
        Response::Single(ResponseOutput::Failure(failure))
    }
}
