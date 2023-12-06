use js_sys::Promise;
use wasm_bindgen::{prelude::wasm_bindgen, JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, Response};

use crate::{options::FetchOptions, Error};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = fetch)]
    fn fetch_with_request_and_init(input: &Request, init: &RequestInit) -> Promise;
}

fn js_fetch(request: &Request, options: Option<FetchOptions>) -> Promise {
    let global = js_sys::global();

    let init = options.map(Into::into).unwrap_or_default();

    if let Ok(true) = js_sys::Reflect::has(&global, &JsValue::from_str("ServiceWorkerGlobalScope"))
    {
        global
            .unchecked_into::<web_sys::ServiceWorkerGlobalScope>()
            .fetch_with_request_and_init(request, &init)
    } else {
        fetch_with_request_and_init(request, &init)
    }
}

pub async fn fetch(request: &Request, options: Option<FetchOptions>) -> Result<Response, Error> {
    let js_response = JsFuture::from(js_fetch(request, options))
        .await
        .map_err(Error::js_error)?;

    Ok(js_response.unchecked_into())
}
