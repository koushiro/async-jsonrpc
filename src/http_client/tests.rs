use hyper::{
    body::{Body, HttpBody as _},
    service::{make_service_fn, service_fn},
    Method, Request as HttpRequest, Response as HttpResponse,
};

use super::*;

#[test]
fn http_basic_auth() {
    let builder = HttpTransportBuilder::new().basic_auth("username", Some("password"));
    let basic_auth = builder.headers.get(header::AUTHORIZATION).unwrap();
    assert_eq!(
        basic_auth,
        HeaderValue::from_static("Basic dXNlcm5hbWU6cGFzc3dvcmQ=")
    );

    let builder = HttpTransportBuilder::new().basic_auth("username", Option::<String>::None);
    let basic_auth = builder.headers.get(header::AUTHORIZATION).unwrap();
    assert_eq!(basic_auth, HeaderValue::from_static("Basic dXNlcm5hbWU6"));

    let builder = HttpTransportBuilder::new().basic_auth("", Some("password"));
    let basic_auth = builder.headers.get(header::AUTHORIZATION).unwrap();
    assert_eq!(basic_auth, HeaderValue::from_static("Basic OnBhc3N3b3Jk"));
}

#[test]
fn http_bearer_auth() {
    let builder = HttpTransportBuilder::new().bearer_auth("Hold my bear");
    let bearer_auth = builder.headers.get(header::AUTHORIZATION).unwrap();
    assert_eq!(bearer_auth, HeaderValue::from_static("Bearer Hold my bear"));
}

async fn server(req: HttpRequest<Body>) -> hyper::Result<HttpResponse<Body>> {
    assert_eq!(req.method(), &Method::POST);

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
            Ok(HttpResponse::new(response.into()))
        }
        "/v2_params" => {
            let expected = r#"{"jsonrpc":"2.0","method":"bar","params":[],"id":1}"#;
            assert_eq!(std::str::from_utf8(&content), Ok(expected));
            let response = r#"{"jsonrpc":"2.0","id":1,"result":"y"}"#;
            Ok(HttpResponse::new(response.into()))
        }
        "/v2_batch" => {
            let expected = r#"[{"jsonrpc":"2.0","method":"foo","id":1},{"jsonrpc":"2.0","method":"bar","params":[],"id":2}]"#;
            assert_eq!(std::str::from_utf8(&content), Ok(expected));
            let response =
                r#"[{"jsonrpc":"2.0","id":1,"result":"x"},{"jsonrpc":"2.0","id":2,"result":"y"}]"#;
            Ok(HttpResponse::new(response.into()))
        }
        _ => unreachable!(),
    }
}

#[tokio::test]
async fn make_jsonrpc_request() {
    let addr = "127.0.0.1:8080";

    let service = make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(server)) });
    let server = hyper::Server::bind(&addr.parse().unwrap()).serve(service);
    tokio::spawn(server);

    {
        let client = HttpTransport::new(format!("http://{}/v2_no_params", addr)).unwrap();
        let response = client.send("foo", None).await.unwrap();
        assert_eq!(
            response,
            Success {
                jsonrpc: Version::V2_0,
                result: Value::String("x".to_string()),
                id: Id::Num(1),
            }
            .into()
        );
    }

    {
        let client = HttpTransport::new(format!("http://{}/v2_params", addr)).unwrap();
        let response = client
            .send("bar", Some(Params::Array(vec![])))
            .await
            .unwrap();
        assert_eq!(response, Success::new("y".into(), 1.into()).into());
    }

    {
        let client = HttpTransport::new(format!("http://{}/v2_batch", addr)).unwrap();
        let response = client
            .send_batch(vec![("foo", None), ("bar", Some(Params::Array(vec![])))])
            .await
            .unwrap();
        assert_eq!(
            response,
            Response::Batch(vec![
                Output::Success(Success::new("x".into(), 1.into())),
                Output::Success(Success::new("y".into(), 2.into())),
            ])
        );
    }
}
