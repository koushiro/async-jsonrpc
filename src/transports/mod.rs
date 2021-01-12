mod http;

pub use self::http::*;

use serde::de::DeserializeOwned;

use crate::errors::Result;
use crate::types::*;

/// Transport implementation.
#[async_trait::async_trait]
pub trait Transport {
    /// Prepare serializable RPC call for given method with parameters.
    fn prepare<M: Into<String>>(&self, method: M, params: Params) -> (RequestId, Call);

    /// Execute prepared RPC call.
    async fn execute(&self, id: RequestId, request: &Request) -> Result<Response>;

    /// Send remote method with given parameters.
    async fn send<M, T>(&self, method: M, params: Params) -> Result<T>
    where
        M: Into<String> + Send,
        T: DeserializeOwned,
    {
        let (id, call) = self.prepare(method, params);
        let request = Request::Single(call);
        debug!(
            "Request: {}",
            serde_json::to_string(&request).expect("Serialize `Request` never fails")
        );

        let response = self.execute(id, &request).await?;
        debug!(
            "Response: {}",
            serde_json::to_string(&response).expect("Serialize `Response` never fails")
        );
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
#[async_trait::async_trait]
pub trait BatchTransport: Transport {
    /// Execute a batch of prepared RPC calls.
    async fn execute_batch<I>(&self, requests: I) -> Result<Response>
    where
        I: IntoIterator<Item = (RequestId, Call)> + Send,
        I::IntoIter: Send,
    {
        let mut iter = requests.into_iter();
        let (id, first): (RequestId, Option<Call>) = match iter.next() {
            Some(request) => (request.0, Some(request.1)),
            None => (0, None),
        };
        let calls = first
            .into_iter()
            .chain(iter.map(|request| request.1))
            .collect::<Vec<_>>();
        let request = Request::Batch(calls);
        debug!(
            "Request: {}",
            serde_json::to_string(&request).expect("Serialize `Request` never fails")
        );

        self.execute(id, &request).await
    }

    /// Send a batch of RPC calls with the given method and parameters.
    async fn send_batch<I, M>(&self, method_and_params: I) -> Result<Vec<Result<Value>>>
    where
        I: IntoIterator<Item = (M, Params)> + Send,
        I::IntoIter: Send,
        M: Into<String>,
    {
        let requests = method_and_params
            .into_iter()
            .map(|(method, params)| self.prepare(method, params));

        let response = self.execute_batch(requests).await?;
        debug!(
            "Response: {}",
            serde_json::to_string(&response).expect("Serialize `Response` never fails")
        );
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

    /// Send a batch of RPC calls with the same method and the given parameters.
    /// Once a request result returns an error, which will be returned directly.
    async fn send_batch_same<I, M, T>(&self, method: M, batch_params: I) -> Result<Vec<T>>
    where
        I: IntoIterator<Item = Params> + Send,
        I::IntoIter: Send,
        M: Into<String> + Send,
        T: DeserializeOwned,
    {
        let method = method.into();
        let calls = batch_params
            .into_iter()
            .map(|params| self.prepare(method.clone(), params));

        let response = self.execute_batch(calls).await?;
        debug!(
            "Response: {}",
            serde_json::to_string(&response).expect("Serialize `Response` never fails")
        );
        let values = match response {
            Response::Single(_) => panic!("Expected batch, got single"),
            Response::Batch(outputs) => outputs,
        };
        let mut results = Vec::with_capacity(values.len());
        for value in values {
            let value = match value {
                ResponseOutput::Success(success) => success.result,
                ResponseOutput::Failure(failure) => return Err(failure.error.into()),
            };
            let result = serde_json::from_value(value).expect("Deserialize `Value` never fails");
            results.push(result);
        }
        Ok(results)
    }
}

/// The type of stream pub-sub transport returns.
pub type NotificationStream<T> = futures::stream::BoxStream<'static, T>;

/// A transport implementation supporting pub sub subscriptions.
pub trait PubsubTransport: Transport {
    /// Add a subscription to this transport
    fn subscribe<T>(&self, id: SubscriptionId) -> NotificationStream<T>
    where
        T: DeserializeOwned;

    /// Remove a subscription from this transport
    fn unsubscribe(&self, id: SubscriptionId);
}
