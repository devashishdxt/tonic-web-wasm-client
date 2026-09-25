//! `fetch` rejecting with values that are not errors, as a page that wraps `fetch` can make it do.

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

/// Calls `Echo` while `globalThis.fetch` is replaced by `replacement`, a JS expression.
async fn echo_with_fetch(replacement: &str) -> Result<String, tonic::Status> {
    let global = js_sys::global();
    let fetch = js_sys::Reflect::get(&global, &"fetch".into()).unwrap();
    js_sys::eval(&format!("globalThis.fetch = {replacement}")).unwrap();

    let result = build_client()
        .echo(EchoRequest {
            message: "John".to_string(),
        })
        .await;

    js_sys::Reflect::set(&global, &"fetch".into(), &fetch).unwrap();
    result.map(|response| response.into_inner().message)
}

#[wasm_bindgen_test]
async fn test_fetch_rejecting_with_null() {
    let error = echo_with_fetch("() => Promise.reject(null)")
        .await
        .unwrap_err();

    assert!(error.message().contains("null"), "{error:?}");
}

#[wasm_bindgen_test]
async fn test_fetch_rejecting_with_undefined() {
    let error = echo_with_fetch("() => Promise.reject(undefined)")
        .await
        .unwrap_err();

    assert!(error.message().contains("undefined"), "{error:?}");
}

#[wasm_bindgen_test]
async fn test_fetch_rejecting_with_an_object_without_to_string() {
    let result = echo_with_fetch("() => Promise.reject(Object.create(null))").await;

    assert!(result.is_err());
}
