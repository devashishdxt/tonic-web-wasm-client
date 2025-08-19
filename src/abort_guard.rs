use crate::Error;
use web_sys::{AbortController, AbortSignal};

/// A guard that cancels a fetch request when dropped.
pub(crate) struct AbortGuard {
    ctrl: AbortController,
}

impl AbortGuard {
    pub(crate) fn new() -> Result<Self, crate::Error> {
        Ok(AbortGuard {
            ctrl: AbortController::new().map_err(Error::js_error)?,
        })
    }

    pub(crate) fn signal(&self) -> AbortSignal {
        self.ctrl.signal()
    }
}

impl Drop for AbortGuard {
    fn drop(&mut self) {
        self.ctrl.abort();
    }
}
