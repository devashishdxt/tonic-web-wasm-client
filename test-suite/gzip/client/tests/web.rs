use client::proto::{echo_client::EchoClient, EchoRequest};
use tonic::codegen::CompressionEncoding;
use tonic_web_wasm_client::Client;
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
wasm_bindgen_test_configure!(run_in_browser);

fn build_client() -> EchoClient<Client> {
    let base_url = "http://localhost:50051".to_string();
    let wasm_client = Client::new(base_url);

    EchoClient::new(wasm_client).accept_compressed(CompressionEncoding::Gzip)
}

#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "wasi", target_env = "p2"),
    wstd::test
)]
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

#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "wasi", target_env = "p2"),
    wstd::test
)]
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

#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "wasi", target_env = "p2"),
    wstd::test
)]
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
