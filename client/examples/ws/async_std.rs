use std::time::Duration;

use async_jsonrpc_client::{BatchTransport, PubsubTransport, ResponseObj, Transport, WsClient, WsClientError};

#[async_std::main]
async fn main() -> Result<(), WsClientError> {
    env_logger::init();

    let client = WsClient::new("wss://rpc.polkadot.io").await?;

    let response = client.request("system_chain", None).await?;
    log::info!("Response: {}", ResponseObj::Single(response));

    let response = client
        .request_batch(vec![("system_chain", None), ("system_chainType", None)])
        .await?;
    log::info!("Response: {}", ResponseObj::Batch(response));

    let (id, mut rx) = client.subscribe("chain_subscribeNewHead", None).await?;
    log::info!("Subscription ID: {}", id);

    let client_clone = client.clone();
    async_std::task::spawn(async move {
        async_std::task::sleep(Duration::from_secs(20)).await;
        let _ = client_clone.unsubscribe("chain_unsubscribeNewHead", id).await;
    });

    while let Some(notification) = rx.next().await {
        log::info!("Subscription Notification: {}", notification);
    }

    Ok(())
}
