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
    ///
    /// This is fetch's default mode.
    #[default]
    Cors,

    /// Restricts requests to using CORS-safelisted methods and CORS-safelisted request-headers. Upon success, fetch
    /// will return an opaque filtered response.
    ///
    /// A grpc-web call cannot work in this mode across origins: its headers are not CORS-safelisted and the response
    /// is opaque.
    NoCors,

    /// This is a special mode used only when navigating between documents.
    #[deprecated(note = "fetch rejects every request whose mode is `navigate`")]
    Navigate,
}

impl From<Mode> for RequestMode {
    fn from(value: Mode) -> Self {
        match value {
            Mode::SameOrigin => RequestMode::SameOrigin,
            Mode::Cors => RequestMode::Cors,
            Mode::NoCors => RequestMode::NoCors,
            #[allow(deprecated)]
            Mode::Navigate => RequestMode::Navigate,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_cors_like_fetch() {
        assert_eq!(RequestMode::from(Mode::default()), RequestMode::Cors);
    }
}
