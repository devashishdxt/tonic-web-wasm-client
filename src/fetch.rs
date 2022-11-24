use js_sys::Promise;
use wasm_bindgen::{prelude::wasm_bindgen, JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, Response};

use crate::Error;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = fetch)]
    fn fetch_with_request(input: &Request) -> Promise;
}

fn js_fetch(request: &Request) -> Promise {
    let global = js_sys::global();

    if let Ok(true) = js_sys::Reflect::has(&global, &JsValue::from_str("ServiceWorkerGlobalScope"))
    {
        global
            .unchecked_into::<web_sys::ServiceWorkerGlobalScope>()
            .fetch_with_request(request)
    } else {
        // browser
        fetch_with_request(request)
    }
}

pub async fn fetch(request: &Request) -> Result<Response, Error> {
    let js_response = JsFuture::from(js_fetch(request))
        .await
        .map_err(Error::js_error)?;

    Ok(js_response.unchecked_into())
}
