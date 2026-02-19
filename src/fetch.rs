use js_sys::Promise;
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
        .map_err(Error::js_error)?;

    Ok(js_response.unchecked_into())
}
