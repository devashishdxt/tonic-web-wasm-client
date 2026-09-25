use std::time::Duration;

use js_sys::Function;
use wasm_bindgen::{
    JsCast, JsValue,
    prelude::{Closure, wasm_bindgen},
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
            timeout_millis(timeout),
        );
        if let Some((id, _)) = self.timeout.replace((timeout, abort)) {
            clear_timeout(id);
        }
    }
}

/// The `setTimeout` delay for `timeout`. The delay is a signed 32-bit number of milliseconds, and
/// browsers fire at once for a larger one, so a longer timeout is clamped to the largest delay,
/// about 24.8 days.
fn timeout_millis(timeout: Duration) -> i32 {
    timeout.as_millis().try_into().unwrap_or(i32::MAX)
}

impl Drop for AbortGuard {
    fn drop(&mut self) {
        self.ctrl.abort();

        if let Some((id, _)) = self.timeout.take() {
            clear_timeout(id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timeout_is_clamped_to_the_largest_set_timeout_delay() {
        let largest = Duration::from_millis(i32::MAX as u64);

        assert_eq!(timeout_millis(Duration::from_millis(1500)), 1500);
        assert_eq!(timeout_millis(largest), i32::MAX);
        assert_eq!(timeout_millis(largest + Duration::from_millis(1)), i32::MAX);
        assert_eq!(timeout_millis(Duration::MAX), i32::MAX);
    }
}
