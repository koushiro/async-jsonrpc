/*
use async_jsonrpc_client::{
    Params, PubsubTransport, SubscriptionId, Transport, Value, WebSocketTransport,
};
use futures::stream::StreamExt;

#[tokio::main]
async fn main() {
    env_logger::init();

    let ws = WebSocketTransport::new("ws://127.0.0.1:1234/rpc/v0");
    // Filecoin.Version need read permission
    let version: Value = ws
        .send("Filecoin.Version", Params::Array(vec![]))
        .await
        .unwrap();
    println!("Version: {:?}", version);

    // lotus auth create-token --perm admin
    let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJBbGxvdyI6WyJyZWFkIiwid3JpdGUiLCJzaWduIiwiYWRtaW4iXX0.zKxsT3HxoYdjy6A8cF1Q3qvJEbCxJb3PcAZ_CM-sf9I";
    let ws = WebSocketTransport::new_with_bearer_auth("ws://127.0.0.1:1234/rpc/v0", token);
    // Filecoin.LogList need write permission
    let log_list: Value = ws
        .send("Filecoin.LogList", Params::Array(vec![]))
        .await
        .unwrap();
    println!("LogList: {:?}", log_list);

    let ws = WebSocketTransport::new("ws://127.0.0.1:1234/rpc/v0");
    let id: SubscriptionId = ws
        .send("Filecoin.SyncIncomingBlocks", Params::Array(vec![]))
        .await
        .unwrap();
    println!("Subscription Id: {}", id);
    let mut stream = ws.subscribe::<Value>(id);
    while let Some(value) = stream.next().await {
        println!("Block: {:?}", value);
    }
}
*/

fn main() {}
