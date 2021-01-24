/*
use async_jsonrpc_client::{HttpReqwestTransport, Params, Transport, Value};

#[tokio::main]
async fn main() {
    env_logger::init();

    let http = HttpReqwestTransport::new("http://127.0.0.1:1234/rpc/v0");
    // Filecoin.Version need read permission
    let version: Value = http
        .send("Filecoin.Version", Params::Array(vec![]))
        .await
        .unwrap();
    println!("Version: {:?}", version);

    // lotus auth create-token --perm admin
    let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJBbGxvdyI6WyJyZWFkIiwid3JpdGUiLCJzaWduIiwiYWRtaW4iXX0.V82x4rrMmyzgLhW0jeBCL6FVN8I6iSnB0Dc05xeZjVE";
    let http = HttpReqwestTransport::new_with_bearer_auth("http://127.0.0.1:1234/rpc/v0", token);
    // Filecoin.LogList need write permission
    let log_list: Value = http
        .send("Filecoin.LogList", Params::Array(vec![]))
        .await
        .unwrap();
    println!("LogList: {:?}", log_list);
}
*/

fn main() {}
