use web_sys::RequestCredentials;

/// Request's credential mode
#[derive(Debug, Clone, Copy, Default)]
pub enum Credentials {
    /// Excludes credentials from this request, and causes any credentials sent back in the response to be ignored.
    Omit,
    /// Include credentials with requests made to same-origin URLs, and use any credentials sent back in responses from
    /// same-origin URLs.
    #[default]
    SameOrigin,
    /// Always includes credentials with this request, and always use any credentials sent back in the response.
    Include,
}

impl From<Credentials> for RequestCredentials {
    fn from(credentials: Credentials) -> Self {
        match credentials {
            Credentials::Omit => RequestCredentials::Omit,
            Credentials::SameOrigin => RequestCredentials::SameOrigin,
            Credentials::Include => RequestCredentials::Include,
        }
    }
}
