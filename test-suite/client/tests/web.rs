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
