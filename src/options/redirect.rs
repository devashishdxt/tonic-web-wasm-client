use web_sys::RequestRedirect;

/// Request's redirect mode
#[derive(Debug, Clone, Copy, Default)]
pub enum Redirect {
    /// Follow all redirects incurred when fetching a resource.
    #[default]
    Follow,
    /// Return a network error when a request is met with a redirect.
    Error,
    /// Retrieves an opaque-redirect filtered response when a request is met with a redirect, to allow a service worker
    /// to replay the redirect offline. The response is otherwise indistinguishable from a network error, to not violate
    /// atomic HTTP redirect handling.
    Manual,
}

impl From<Redirect> for RequestRedirect {
    fn from(value: Redirect) -> Self {
        match value {
            Redirect::Follow => RequestRedirect::Follow,
            Redirect::Error => RequestRedirect::Error,
            Redirect::Manual => RequestRedirect::Manual,
        }
    }
}
