use std::time::Duration;

use client::proto::{echo_client::EchoClient, EchoRequest};
use tonic::Code;
use tonic_web_wasm_client::{options::FetchOptions, Client};
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

fn build_client() -> EchoClient<Client> {
    let base_url = "http://localhost:50051".to_string();

    let mut wasm_client = Client::new(base_url);
    wasm_client.with_options(FetchOptions::default().timeout(Duration::from_secs(2)));

    EchoClient::new(wasm_client)
}

#[wasm_bindgen_test]
async fn test_echo() {
    let mut client = build_client();

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
async fn test_echo_timeout() {
    let mut client = build_client();

    let error = client
        .echo_timeout(EchoRequest {
            message: "John".to_string(),
        })
        .await
        .unwrap_err();

    assert_eq!(error.code(), Code::DeadlineExceeded);
}

#[wasm_bindgen_test]
async fn test_echo_stream() {
    let mut client = build_client();

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

    let response = stream_response.message().await.expect("stream message");
    assert!(response.is_none());
}

#[wasm_bindgen_test]
async fn test_infinite_echo_stream() {
    let mut client = build_client();

    let mut stream_response = client
        .echo_infinite_stream(EchoRequest {
            message: "John".to_string(),
        })
        .await
        .expect("success stream response")
        .into_inner();

    for i in 0..3 {
        let response = stream_response.message().await.expect("stream message");
        assert!(response.is_some(), "{}", i);
        let response = response.unwrap();

        assert_eq!(response.message, format!("echo(John, {})", i + 1));
    }

    let response = stream_response.message().await.expect("stream message");
    assert!(response.is_some());
}

#[wasm_bindgen_test]
async fn test_error_response() {
    let mut client = build_client();

    let error = client
        .echo_error_response(EchoRequest {
            message: "John".to_string(),
        })
        .await
        .unwrap_err();

    assert_eq!(error.code(), Code::Unauthenticated);
}
