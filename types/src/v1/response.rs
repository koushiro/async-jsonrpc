#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
use core::{fmt, marker::PhantomData};

use serde::{
    de::{self, DeserializeOwned},
    Deserialize, Serialize,
};
use serde_json::Value;

use crate::v1::{Error, ErrorCode, Id};

/// JSON-RPC 1.0 Response Object.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum ResponseObj<T = Value> {
    /// Single response
    Single(Response<T>),
    /// Batch of responses (response to batch request)
    Batch(BatchResponse<T>),
}

impl<T: Serialize> fmt::Display for ResponseObj<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let json = serde_json::to_string(self).expect("`ResponseObj` is serializable");
        write!(f, "{}", json)
    }
}

/// Represents JSON-RPC 1.0 batch response.
pub type BatchResponse<T = Value> = Vec<Response<T>>;

/// Represents JSON-RPC 1.0 success / failure response.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Response<T = Value> {
    /// Successful execution result.
    pub result: Option<T>,
    /// Failed execution error.
    pub error: Option<Error>,
    /// Correlation id.
    ///
    /// It **MUST** be the same as the value of the id member in the Request Object.
    ///
    /// If there was an error in detecting the id in the Request object (e.g. Parse error/Invalid Request),
    /// it **MUST** be Null.
    pub id: Option<Id>,
}

impl<T: Serialize> fmt::Display for Response<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let json = serde_json::to_string(self).expect("`Response` is serializable");
        write!(f, "{}", json)
    }
}

impl<'de, T: Deserialize<'de>> de::Deserialize<'de> for Response<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        use self::response_field::{Field, FIELDS};

        struct Visitor<'de, T> {
            marker: PhantomData<Response<T>>,
            lifetime: PhantomData<&'de ()>,
        }
        impl<'de, T: Deserialize<'de>> de::Visitor<'de> for Visitor<'de, T> {
            type Value = Response<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("struct Response")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let mut result = Option::<Option<T>>::None;
                let mut error = Option::<Option<Error>>::None;
                let mut id = Option::<Option<Id>>::None;

                while let Some(key) = de::MapAccess::next_key::<Field>(&mut map)? {
                    match key {
                        Field::Result => {
                            if result.is_some() {
                                return Err(de::Error::duplicate_field("result"));
                            }
                            result = Some(de::MapAccess::next_value::<Option<T>>(&mut map)?)
                        }
                        Field::Error => {
                            if error.is_some() {
                                return Err(de::Error::duplicate_field("error"));
                            }
                            error = Some(de::MapAccess::next_value::<Option<Error>>(&mut map)?)
                        }
                        Field::Id => {
                            if id.is_some() {
                                return Err(de::Error::duplicate_field("id"));
                            }
                            id = Some(de::MapAccess::next_value::<Option<Id>>(&mut map)?)
                        }
                    }
                }

                let result = result.ok_or_else(|| de::Error::missing_field("result"))?;
                let error = error.ok_or_else(|| de::Error::missing_field("error"))?;
                let id = id.ok_or_else(|| de::Error::missing_field("id"))?;
                let (result, error, id) = match (result, error, id) {
                    (Some(value), None, Some(id)) => (Some(value), None, Some(id)),
                    (None, Some(error), id) => (None, Some(error), id),
                    _ => return Err(de::Error::custom("Invalid JSON-RPC 1.0 response")),
                };
                Ok(Response { result, error, id })
            }
        }

        de::Deserializer::deserialize_struct(
            deserializer,
            "Response",
            FIELDS,
            Visitor {
                marker: PhantomData::<Response<T>>,
                lifetime: PhantomData,
            },
        )
    }
}

impl<T: Serialize + DeserializeOwned> Response<T> {
    /// Creates a JSON-RPC 1.0 success response.
    pub fn success(result: T, id: Id) -> Self {
        Self {
            result: Some(result),
            error: None,
            id: Some(id),
        }
    }

    /// Creates a JSON-RPC 1.0 failure response.
    pub fn failure(error: Error, id: Option<Id>) -> Self {
        Self {
            result: None,
            error: Some(error),
            id,
        }
    }

    /// Creates a new failure response output indicating malformed request.
    pub fn invalid_request(id: Option<Id>) -> Self {
        Self::failure(Error::new(ErrorCode::InvalidRequest), id)
    }
}

impl<T: Serialize + DeserializeOwned> From<Response<T>> for Result<T, Error> {
    // Convert into a result.
    // Will be `Ok` if it is a `SuccessResponse` and `Err` if `FailureResponse`.
    fn from(resp: Response<T>) -> Result<T, Error> {
        match (resp.result, resp.error) {
            (Some(result), None) => Ok(result),
            (None, Some(error)) => Err(error),
            _ => unreachable!("Invalid JSON-RPC 1.0 Response"),
        }
    }
}

mod response_field {
    use super::*;

    pub const FIELDS: &[&str] = &["result", "error", "id"];
    pub enum Field {
        Result,
        Error,
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
                "result" => Ok(Field::Result),
                "error" => Ok(Field::Error),
                "id" => Ok(Field::Id),
                _ => Err(de::Error::unknown_field(v, &FIELDS)),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn response_cases() -> Vec<(Response, &'static str)> {
        vec![
            (
                // JSON-RPC 1.0 success response
                Response {
                    result: Some(Value::Bool(true)),
                    error: None,
                    id: Some(Id::Num(1)),
                },
                r#"{"result":true,"error":null,"id":1}"#,
            ),
            (
                // JSON-RPC 1.0 failure response
                Response {
                    result: None,
                    error: Some(Error::parse_error()),
                    id: Some(Id::Num(1)),
                },
                r#"{"result":null,"error":{"code":-32700,"message":"Parse error"},"id":1}"#,
            ),
            (
                // JSON-RPC 1.0 failure response
                Response {
                    result: None,
                    error: Some(Error::parse_error()),
                    id: None,
                },
                r#"{"result":null,"error":{"code":-32700,"message":"Parse error"},"id":null}"#,
            ),
        ]
    }

    #[test]
    fn response_serialization() {
        for (response, expect) in response_cases() {
            let ser = serde_json::to_string(&response).unwrap();
            assert_eq!(ser, expect);
            let de = serde_json::from_str::<Response>(expect).unwrap();
            assert_eq!(de, response);
        }

        // JSON-RPC 1.0 valid response
        let valid_cases = vec![
            r#"{"result":true,"error":null,"id":1}"#,
            r#"{"result":null,"error":{"code": -32700,"message": "Parse error"},"id":1}"#,
        ];
        for case in valid_cases {
            let response = serde_json::from_str::<Response>(case);
            assert!(response.is_ok());
        }

        // JSON-RPC 1.0 invalid response
        let invalid_cases = vec![
            r#"{"result":true,"error":null,"id":1,unknown:[]}"#,
            r#"{"result":true,"error":{"code": -32700,"message": "Parse error"},"id":1}"#,
            r#"{"result":true,"error":{"code": -32700,"message": "Parse error"}}"#,
            r#"{"result":true,"id":1}"#,
            r#"{"error":{"code": -32700,"message": "Parse error"},"id":1}"#,
            r#"{"unknown":[]}"#,
        ];

        for case in invalid_cases {
            let response = serde_json::from_str::<Response>(case);
            assert!(response.is_err());
        }
    }

    #[test]
    fn batch_response_serialization() {
        let batch_response = vec![
            Response {
                result: Some(Value::Bool(true)),
                error: None,
                id: Some(Id::Num(1)),
            },
            Response {
                result: Some(Value::Bool(false)),
                error: None,
                id: Some(Id::Num(2)),
            },
        ];
        let batch_expect = r#"[{"result":true,"error":null,"id":1},{"result":false,"error":null,"id":2}]"#;
        assert_eq!(serde_json::to_string(&batch_response).unwrap(), batch_expect);
        assert_eq!(
            serde_json::from_str::<BatchResponse>(&batch_expect).unwrap(),
            batch_response
        );

        // JSON-RPC 1.0 valid batch response
        let valid_cases = vec![
            r#"[{"result":true,"error":null,"id":1}]"#,
            r#"[{"result":null,"error":{"code": -32700,"message": "Parse error"},"id":1}]"#,
            r#"[{"result":true,"error":null,"id":1}, {"result":null,"error":{"code": -32700,"message": "Parse error"},"id":1}]"#,
        ];
        for case in valid_cases {
            let response = serde_json::from_str::<BatchResponse>(case);
            assert!(response.is_ok());
        }

        // JSON-RPC 1.0 invalid batch response
        let invalid_cases = vec![
            r#"[{"result":true,"error":null,"id":1}"#,
            r#"[{"result":true,"error":{"code": -32700,"message": "Parse error"},"id":1}]"#,
            r#"[{"result":true,"error":{"code": -32700,"message": "Parse error"}}]"#,
            r#"[{"result":true,"id":1}]"#,
            r#"[{"error":{"code": -32700,"message": "Parse error"},"id":1}]"#,
            // r#"[]"#, // empty should be invalid
        ];
        for case in invalid_cases {
            let response = serde_json::from_str::<BatchResponse>(case);
            assert!(response.is_err());
        }
    }
}
