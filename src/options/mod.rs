//! Options for underlying `fetch` call
mod cache;
mod credentials;
mod mode;
mod redirect;
mod referrer_policy;

use web_sys::RequestInit;

pub use self::{
    cache::Cache, credentials::Credentials, mode::Mode, redirect::Redirect,
    referrer_policy::ReferrerPolicy,
};

/// Options for underlying `fetch` call
#[derive(Debug, Clone, Default)]
pub struct FetchOptions {
    /// Request's cache mode
    pub cache: Option<Cache>,
    /// Request's credentials mode
    pub credentials: Option<Credentials>,
    /// Requests's integrity
    pub integrity: Option<String>,
    /// Request's mode
    pub mode: Option<Mode>,
    /// Request's redirect mode
    pub redirect: Option<Redirect>,
    /// Request's referrer
    pub referrer: Option<String>,
    /// Request's referrer policy
    pub referrer_policy: Option<ReferrerPolicy>,
}

impl FetchOptions {
    /// Create new `Options` with default values
    pub fn new() -> Self {
        Default::default()
    }

    /// Set request's cache mode
    pub fn cache(mut self, cache: Cache) -> Self {
        self.cache = Some(cache);
        self
    }

    /// Set request's credentials mode
    pub fn credentials(mut self, credentials: Credentials) -> Self {
        self.credentials = Some(credentials);
        self
    }

    /// Set request's integrity
    pub fn integrity(mut self, integrity: String) -> Self {
        self.integrity = Some(integrity);
        self
    }

    /// Set request's mode
    pub fn mode(mut self, mode: Mode) -> Self {
        self.mode = Some(mode);
        self
    }

    /// Set request's redirect mode
    pub fn redirect(mut self, redirect: Redirect) -> Self {
        self.redirect = Some(redirect);
        self
    }

    /// Set request's referrer
    pub fn referrer(mut self, referrer: String) -> Self {
        self.referrer = Some(referrer);
        self
    }

    /// Set request's referrer policy
    pub fn referrer_policy(mut self, referrer_policy: ReferrerPolicy) -> Self {
        self.referrer_policy = Some(referrer_policy);
        self
    }
}

impl From<FetchOptions> for RequestInit {
    fn from(value: FetchOptions) -> Self {
        let mut init = RequestInit::new();

        if let Some(cache) = value.cache {
            init.cache(cache.into());
        }

        if let Some(credentials) = value.credentials {
            init.credentials(credentials.into());
        }

        if let Some(ref integrity) = value.integrity {
            init.integrity(integrity);
        }

        if let Some(mode) = value.mode {
            init.mode(mode.into());
        }

        if let Some(redirect) = value.redirect {
            init.redirect(redirect.into());
        }

        if let Some(ref referrer) = value.referrer {
            init.referrer(referrer);
        }

        if let Some(referrer_policy) = value.referrer_policy {
            init.referrer_policy(referrer_policy.into());
        }

        init
    }
}
