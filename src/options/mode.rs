use web_sys::RequestMode;

/// Request's mode
#[derive(Debug, Clone, Copy, Default)]
pub enum Mode {
    /// Used to ensure requests are made to same-origin URLs. Fetch will return a network error if the request is not
    /// made to a same-origin URL.
    SameOrigin,
    /// For requests whose response tainting gets set to "cors", makes the request a CORS request — in which case, fetch
    /// will return a network error if the requested resource does not understand the CORS protocol, or if the requested
    /// resource is one that intentionally does not participate in the CORS protocol.
    Cors,
    /// Restricts requests to using CORS-safelisted methods and CORS-safelisted request-headers. Upon success, fetch
    /// will return an opaque filtered response.
    #[default]
    NoCors,
    /// This is a special mode used only when navigating between documents.
    Navigate,
}

impl From<Mode> for RequestMode {
    fn from(value: Mode) -> Self {
        match value {
            Mode::SameOrigin => RequestMode::SameOrigin,
            Mode::Cors => RequestMode::Cors,
            Mode::NoCors => RequestMode::NoCors,
            Mode::Navigate => RequestMode::Navigate,
        }
    }
}
