use js_sys::Promise;
use tonic::Status;
use wasm_bindgen::{JsCast, JsValue, prelude::wasm_bindgen};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, Response};

use crate::Error;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = fetch)]
    fn fetch_with_request_and_init(input: &Request, init: &RequestInit) -> Promise;
}

fn js_fetch(request: &Request, init: &RequestInit) -> Promise {
    let global = js_sys::global();
    let key = JsValue::from_str("ServiceWorkerGlobalScope");

    match js_sys::Reflect::has(&global, &key) {
        Ok(true) => global
            .unchecked_into::<web_sys::ServiceWorkerGlobalScope>()
            .fetch_with_request_and_init(request, init),
        _ => fetch_with_request_and_init(request, init),
    }
}

pub async fn fetch(request: &Request, init: &RequestInit) -> Result<Response, Error> {
    let js_response = JsFuture::from(js_fetch(request, init))
        .await
        .map_err(|error| match Error::js_error(error) {
            // fetch rejects when no response arrives: the connection or DNS lookup failed, CORS
            // blocked the response, ... For gRPC that is `Unavailable`, as with tonic's own
            // transport. The message is kept, so the cause stays visible.
            error @ Error::JsError(_) => {
                Error::TonicStatusError(Status::unavailable(error.to_string()))
            }
            error => error,
        })?;

    Ok(js_response.unchecked_into())
}
