#[cfg(feature = "http")]
mod http;
// #[cfg(feature = "ws")]
// mod ws;

#[cfg(feature = "http")]
pub use self::http::*;
// #[cfg(feature = "ws")]
// pub use self::ws::*;

use futures::stream::Stream;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::errors::Result;
use crate::types::*;

/// Transport implementation.
#[async_trait::async_trait(?Send)]
pub trait Transport: Clone {
    /// Prepare serializable RPC call for given method with parameters.
    fn prepare<M: Into<String>>(&self, method: M, params: Params) -> Call;

    /// Execute prepared RPC call.
    async fn execute(&self, request: &Request) -> Result<Response>;

    /// Send remote method with given parameters.
    async fn send<M, T>(&self, method: M, params: Params) -> Result<T>
    where
        M: Into<String>,
        T: DeserializeOwned,
    {
        let call = self.prepare(method, params);
        let request = Request::Single(call);
        let response = self.execute(&request).await?;
        match response {
            Response::Single(ResponseOutput::Success(success)) => {
                Ok(serde_json::from_value(success.result)?)
            }
            Response::Single(ResponseOutput::Failure(failure)) => Err(failure.error.into()),
            Response::Batch(_) => panic!("Expected single, got batch"),
        }
    }
}

/// A transport implementation supporting batch requests
#[async_trait::async_trait(?Send)]
pub trait BatchTransport: Transport {
    /// Execute a batch of prepared RPC calls.
    async fn execute_batch<I>(&self, requests: I) -> Result<Vec<Result<Value>>>
    where
        I: IntoIterator<Item = Call>,
    {
        let request = Request::Batch(requests.into_iter().collect::<Vec<_>>());
        let response = self.execute(&request).await?;
        match response {
            Response::Single(_) => panic!("Expected batch, got single"),
            Response::Batch(outputs) => Ok(outputs
                .into_iter()
                .map(|output| match output {
                    ResponseOutput::Success(success) => Ok(success.result),
                    ResponseOutput::Failure(failure) => Err(failure.error.into()),
                })
                .collect::<Vec<_>>()),
        }
    }

    /// Send a batch of RPC calls with the given method and parameters.
    async fn send_batch<I, M>(&self, method_and_params: I) -> Result<Vec<Result<Value>>>
    where
        I: IntoIterator<Item = (M, Params)>,
        M: Into<String>,
    {
        let calls = method_and_params
            .into_iter()
            .map(|(method, params)| self.prepare(method, params))
            .collect::<Vec<_>>();
        self.execute_batch(calls).await
    }

    /// Send a batch of RPC calls with the same method and the given parameters.
    async fn send_batch_same<I, M, T>(&self, method: M, batch_params: I) -> Result<Vec<T>>
    where
        I: IntoIterator<Item = Params>,
        M: Into<String>,
        T: DeserializeOwned,
    {
        let method = method.into();
        let calls = batch_params
            .into_iter()
            .map(|params| self.prepare(method.clone(), params))
            .collect::<Vec<_>>();
        let values = self.execute_batch(calls).await?;

        let mut results = Vec::with_capacity(values.len());
        for value in values {
            let value = value?;
            let result = serde_json::from_value(value)?;
            results.push(result);
        }
        Ok(results)
    }
}

/// A transport implementation supporting pub sub subscriptions.
#[async_trait::async_trait]
pub trait DuplexTransport: Transport {
    /// The type of stream this transport returns
    type NotificationStream: Stream<Item = Result<Value>>;

    /// Add a subscription to this transport
    async fn subscribe(&self, id: &SubscriptionId) -> Self::NotificationStream;

    /// Remove a subscription from this transport
    fn unsubscribe(&self, id: &SubscriptionId);
}
