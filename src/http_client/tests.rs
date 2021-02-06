use super::*;

#[test]
fn http_header() {
    use http::header::{self, HeaderValue};

    // basic auth
    let builder = HttpClientBuilder::new().basic_auth("username", Some("password"));
    let basic_auth = builder.headers.get(header::AUTHORIZATION).unwrap();
    assert_eq!(basic_auth, HeaderValue::from_static("Basic dXNlcm5hbWU6cGFzc3dvcmQ="));
    let builder = HttpClientBuilder::new().basic_auth("username", Option::<String>::None);
    let basic_auth = builder.headers.get(header::AUTHORIZATION).unwrap();
    assert_eq!(basic_auth, HeaderValue::from_static("Basic dXNlcm5hbWU6"));
    let builder = HttpClientBuilder::new().basic_auth("", Some("password"));
    let basic_auth = builder.headers.get(header::AUTHORIZATION).unwrap();
    assert_eq!(basic_auth, HeaderValue::from_static("Basic OnBhc3N3b3Jk"));

    // bearer auth
    let builder = HttpClientBuilder::new().bearer_auth("Hold my bear");
    let bearer_auth = builder.headers.get(header::AUTHORIZATION).unwrap();
    assert_eq!(bearer_auth, HeaderValue::from_static("Bearer Hold my bear"));
}

#[cfg(feature = "http-async-std")]
async fn server(addr: &str) -> std::io::Result<()> {
    let mut server = tide::new();

    async fn v2_no_params(mut req: tide::Request<()>) -> tide::Result {
        let got = req.body_string().await.unwrap();
        let expected = r#"{"jsonrpc":"2.0","method":"foo","id":1}"#;
        assert_eq!(got, expected);
        let response = r#"{"jsonrpc":"2.0","id":1,"result":"x"}"#;
        Ok(tide::Response::from(response))
    }
    async fn v2_params(mut req: tide::Request<()>) -> tide::Result {
        let got = req.body_string().await.unwrap();
        let expected = r#"{"jsonrpc":"2.0","method":"bar","params":[],"id":1}"#;
        assert_eq!(got, expected);
        let response = r#"{"jsonrpc":"2.0","id":1,"result":"y"}"#;
        Ok(tide::Response::from(response))
    }
    async fn v2_batch(mut req: tide::Request<()>) -> tide::Result {
        let got = req.body_string().await.unwrap();
        let expected =
            r#"[{"jsonrpc":"2.0","method":"foo","id":1},{"jsonrpc":"2.0","method":"bar","params":[],"id":2}]"#;
        assert_eq!(got, expected);
        let response = r#"[{"jsonrpc":"2.0","id":1,"result":"x"},{"jsonrpc":"2.0","id":2,"result":"y"}]"#;
        Ok(tide::Response::from(response))
    }

    server.at("/v2_no_params").post(v2_no_params);
    server.at("/v2_params").post(v2_params);
    server.at("/v2_batch").post(v2_batch);
    server.listen(addr).await
}

#[cfg(feature = "http-async-std")]
#[async_std::test]
async fn make_jsonrpc_request() {
    let addr = "127.0.0.1:8080";
    async_std::task::spawn(server(addr));

    {
        let client = HttpClient::new(format!("http://{}/v2_no_params", addr)).unwrap();
        let response = client.request("foo", None).await.unwrap();
        assert_eq!(response, Output::success(Value::String("x".to_string()), 1.into()));
    }

    {
        let client = HttpClient::new(format!("http://{}/v2_params", addr)).unwrap();
        let response = client.request("bar", Some(Params::Array(vec![]))).await.unwrap();
        assert_eq!(response, Output::success("y".into(), 1.into()));
    }

    {
        let client = HttpClient::new(format!("http://{}/v2_batch", addr)).unwrap();
        let response = client
            .request_batch(vec![("foo", None), ("bar", Some(Params::Array(vec![])))])
            .await
            .unwrap();
        assert_eq!(
            response,
            vec![
                Output::success("x".into(), 1.into()),
                Output::success("y".into(), 2.into()),
            ]
        );
    }
}

#[cfg(feature = "http-tokio")]
async fn dispatch_fn(req: hyper::Request<hyper::Body>) -> hyper::Result<hyper::Response<hyper::Body>> {
    use hyper::body::HttpBody as _;
    assert_eq!(req.method(), &hyper::Method::POST);

    let path = req.uri().path().to_string();
    let mut content = vec![];
    let mut body = req.into_body();
    while let Some(Ok(chunk)) = body.data().await {
        content.extend(&*chunk);
    }
    match path.as_str() {
        "/v2_no_params" => {
            let expected = r#"{"jsonrpc":"2.0","method":"foo","id":1}"#;
            assert_eq!(std::str::from_utf8(&content), Ok(expected));
            let response = r#"{"jsonrpc":"2.0","id":1,"result":"x"}"#;
            Ok(hyper::Response::new(response.into()))
        }
        "/v2_params" => {
            let expected = r#"{"jsonrpc":"2.0","method":"bar","params":[],"id":1}"#;
            assert_eq!(std::str::from_utf8(&content), Ok(expected));
            let response = r#"{"jsonrpc":"2.0","id":1,"result":"y"}"#;
            Ok(hyper::Response::new(response.into()))
        }
        "/v2_batch" => {
            let expected =
                r#"[{"jsonrpc":"2.0","method":"foo","id":1},{"jsonrpc":"2.0","method":"bar","params":[],"id":2}]"#;
            assert_eq!(std::str::from_utf8(&content), Ok(expected));
            let response = r#"[{"jsonrpc":"2.0","id":1,"result":"x"},{"jsonrpc":"2.0","id":2,"result":"y"}]"#;
            Ok(hyper::Response::new(response.into()))
        }
        _ => unreachable!(),
    }
}

#[cfg(feature = "http-tokio")]
#[tokio::test]
async fn make_jsonrpc_request() {
    use hyper::service::{make_service_fn, service_fn};

    let addr = "127.0.0.1:8080";
    let service = make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(dispatch_fn)) });
    let server = hyper::Server::bind(&addr.parse().unwrap()).serve(service);
    tokio::spawn(server);

    {
        let client = HttpClient::new(format!("http://{}/v2_no_params", addr)).unwrap();
        let response = client.request("foo", None).await.unwrap();
        assert_eq!(response, Output::success(Value::String("x".to_string()), 1.into()));
    }

    {
        let client = HttpClient::new(format!("http://{}/v2_params", addr)).unwrap();
        let response = client.request("bar", Some(Params::Array(vec![]))).await.unwrap();
        assert_eq!(response, Output::success("y".into(), 1.into()));
    }

    {
        let client = HttpClient::new(format!("http://{}/v2_batch", addr)).unwrap();
        let response = client
            .request_batch(vec![("foo", None), ("bar", Some(Params::Array(vec![])))])
            .await
            .unwrap();
        assert_eq!(
            response,
            vec![
                Output::success("x".into(), 1.into()),
                Output::success("y".into(), 2.into()),
            ]
        );
    }
}
