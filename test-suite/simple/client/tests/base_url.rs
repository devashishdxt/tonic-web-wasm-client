//! How the request URL is built from the base URL and the method's path.

use std::time::Duration;

use client::proto::{echo_client::EchoClient, EchoRequest};
use tonic_web_wasm_client::{options::FetchOptions, Client};
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

fn build_wasm_client(base_url: &str) -> Client {
    let mut wasm_client = Client::new(base_url.to_string());
    wasm_client.with_options(FetchOptions::default().timeout(Duration::from_secs(2)));
    wasm_client
}

async fn echo(mut client: EchoClient<Client>) -> String {
    client
        .echo(EchoRequest {
            message: "John".to_string(),
        })
        .await
        .expect("success response")
        .into_inner()
        .message
}

#[wasm_bindgen_test]
async fn test_base_url_with_trailing_slash() {
    let client = EchoClient::new(build_wasm_client("http://localhost:50051/"));

    assert_eq!(echo(client).await, "echo(John)");
}

#[wasm_bindgen_test]
async fn test_client_with_origin() {
    let client = EchoClient::with_origin(
        build_wasm_client("http://localhost:50051"),
        "http://example.com".parse().unwrap(),
    );

    assert_eq!(echo(client).await, "echo(John)");
}
