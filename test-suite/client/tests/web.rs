use std::time::Duration;

use client::proto::{echo_client::EchoClient, EchoRequest};
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn test_echo() {
    let mut client = EchoClient::default();

    let response = client
        .echo(EchoRequest {
            message: "John".to_string(),
        })
        .await
        .expect("success response")
        .into_inner();

    assert_eq!(response.message, "echo(John)");
}

#[wasm_bindgen_test]
async fn test_echo_stream() {
    let mut client = EchoClient::default();

    let mut stream_response = client
        .echo_stream(EchoRequest {
            message: "John".to_string(),
        })
        .await
        .expect("success stream response")
        .into_inner();

    for i in 0..3 {
        let response = stream_response.message().await.expect("stream message");
        assert!(response.is_some(), "{}", i);
        let response = response.unwrap();

        assert_eq!(response.message, "echo(John)");
    }
}
