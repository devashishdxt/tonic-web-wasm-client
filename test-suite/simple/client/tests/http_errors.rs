//! Responses that do not come from a gRPC server, and calls that get no response at all.

use std::time::Duration;

use client::proto::{echo_client::EchoClient, EchoRequest};
use tonic::{Code, Status};
use tonic_web_wasm_client::{options::FetchOptions, Client};
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

fn build_client(base_url: &str) -> EchoClient<Client> {
    let mut wasm_client = Client::new(base_url.to_string());
    wasm_client.with_options(FetchOptions::default().timeout(Duration::from_secs(2)));

    EchoClient::new(wasm_client)
}

/// Calls `Echo` through a base URL under which the test server answers every
/// request with `http_status` and a plain text body, like a proxy error page.
async fn echo_answered_with(http_status: u16) -> Status {
    build_client(&format!("http://localhost:50051/status/{http_status}"))
        .echo(EchoRequest {
            message: "John".to_string(),
        })
        .await
        .unwrap_err()
}

#[wasm_bindgen_test]
async fn test_http_503_is_unavailable() {
    assert_eq!(echo_answered_with(503).await.code(), Code::Unavailable);
}

#[wasm_bindgen_test]
async fn test_http_502_is_unavailable() {
    assert_eq!(echo_answered_with(502).await.code(), Code::Unavailable);
}

#[wasm_bindgen_test]
async fn test_http_404_is_unimplemented() {
    assert_eq!(echo_answered_with(404).await.code(), Code::Unimplemented);
}

#[wasm_bindgen_test]
async fn test_http_401_is_unauthenticated() {
    assert_eq!(echo_answered_with(401).await.code(), Code::Unauthenticated);
}

#[wasm_bindgen_test]
async fn test_unreachable_server_is_unavailable() {
    // Nothing listens on this port.
    let error = build_client("http://localhost:50059")
        .echo(EchoRequest {
            message: "John".to_string(),
        })
        .await
        .unwrap_err();

    assert_eq!(error.code(), Code::Unavailable, "{error:?}");
}
