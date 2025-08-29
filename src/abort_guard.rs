use std::time::Duration;

use js_sys::Function;
use wasm_bindgen::{
    prelude::{wasm_bindgen, Closure},
    JsCast, JsValue,
};
use web_sys::{AbortController, AbortSignal};

use crate::Error;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = "setTimeout")]
    fn set_timeout(handler: &Function, timeout: i32) -> JsValue;

    #[wasm_bindgen(js_name = "clearTimeout")]
    fn clear_timeout(handle: JsValue) -> JsValue;
}

/// A guard that cancels a fetch request when dropped.
pub struct AbortGuard {
    ctrl: AbortController,
    timeout: Option<(JsValue, Closure<dyn FnMut()>)>,
}

impl AbortGuard {
    pub fn new() -> Result<Self, Error> {
        Ok(AbortGuard {
            ctrl: AbortController::new().map_err(Error::js_error)?,
            timeout: None,
        })
    }

    pub fn signal(&self) -> AbortSignal {
        self.ctrl.signal()
    }

    pub fn timeout(&mut self, timeout: Duration) {
        let ctrl = self.ctrl.clone();
        let abort = Closure::once(move || {
            ctrl.abort_with_reason(&"tonic_web_wasm_client::Error::TimedOut".into())
        });
        let timeout = set_timeout(
            abort.as_ref().unchecked_ref::<js_sys::Function>(),
            timeout.as_millis().try_into().expect("timeout"),
        );
        if let Some((id, _)) = self.timeout.replace((timeout, abort)) {
            clear_timeout(id);
        }
    }
}

impl Drop for AbortGuard {
    fn drop(&mut self) {
        self.ctrl.abort();

        if let Some((id, _)) = self.timeout.take() {
            clear_timeout(id);
        }
    }
}
