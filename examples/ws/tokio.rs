use async_jsonrpc_client::{BatchTransport, ClientError, PubsubTransport, Response, Transport, WsClient};

#[tokio::main]
async fn main() -> Result<(), ClientError> {
    env_logger::init();

    let client = WsClient::new("wss://rpc.polkadot.io").await?;

    let response = client.request("system_chain", None).await?;
    log::info!("Response: {}", Response::Single(response));

    let response = client
        .request_batch(vec![("system_chain", None), ("system_chainType", None)])
        .await?;
    log::info!("Response: {}", Response::Batch(response));

    let (id, mut rx) = client
        .subscribe("chain_subscribeNewHead", "chain_unsubscribeNewHead", None)
        .await?;
    log::info!("Subscription ID: {}", id);

    /*
    let client_clone = client.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        let _ = client_clone.unsubscribe("chain_unsubscribeNewHead", id).await;
    });
    */

    while let Some(notification) = rx.next().await {
        log::info!("Subscription Notification: {}", notification);
    }

    Ok(())
}
