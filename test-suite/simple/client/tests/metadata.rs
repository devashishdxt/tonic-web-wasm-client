//! Request metadata reaching the server.

use std::time::Duration;

use client::proto::{echo_client::EchoClient, EchoRequest};
use tonic_web_wasm_client::{options::FetchOptions, Client};
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

fn build_client() -> EchoClient<Client> {
    let base_url = "http://localhost:50051".to_string();

    let mut wasm_client = Client::new(base_url);
    wasm_client.with_options(FetchOptions::default().timeout(Duration::from_secs(2)));

    EchoClient::new(wasm_client)
}

/// Sends `x-tag` with `values` and returns what the server received.
async fn echo_x_tag(values: &[&str]) -> String {
    let mut request = tonic::Request::new(EchoRequest {
        message: "x-tag".to_string(),
    });
    for value in values {
        request
            .metadata_mut()
            .append("x-tag", value.parse().unwrap());
    }

    build_client()
        .echo_metadata(request)
        .await
        .expect("success response")
        .into_inner()
        .message
}

#[wasm_bindgen_test]
async fn test_metadata_value() {
    assert_eq!(echo_x_tag(&["a"]).await, "a");
}

#[wasm_bindgen_test]
async fn test_repeated_metadata_sends_every_value() {
    // The browser joins repeated values into one `x-tag: a, b` header line.
    assert_eq!(echo_x_tag(&["a", "b"]).await, "a, b");
}
