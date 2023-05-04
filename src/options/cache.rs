use web_sys::RequestCache;

/// Request's cache mode
#[derive(Debug, Clone, Copy, Default)]
pub enum Cache {
    /// Fetch will inspect the HTTP cache on the way to the network. If the HTTP cache contains a matching fresh
    /// response it will be returned. If the HTTP cache contains a matching stale-while-revalidate response it will be
    /// returned, and a conditional network fetch will be made to update the entry in the HTTP cache. If the HTTP cache
    /// contains a matching stale response, a conditional network fetch will be returned to update the entry in the HTTP
    /// cache. Otherwise, a non-conditional network fetch will be returned to update the entry in the HTTP cache.
    #[default]
    Default,
    /// Fetch behaves as if there is no HTTP cache at all.
    NoStore,
    /// Fetch behaves as if there is no HTTP cache on the way to the network. Ergo, it creates a normal request and
    /// updates the HTTP cache with the response.
    Reload,
    /// Fetch creates a conditional request if there is a response in the HTTP cache and a normal request otherwise. It
    /// then updates the HTTP cache with the response.
    NoCache,
    /// Fetch uses any response in the HTTP cache matching the request, not paying attention to staleness. If there was
    /// no response, it creates a normal request and updates the HTTP cache with the response.
    ForceCache,
    /// Fetch uses any response in the HTTP cache matching the request, not paying attention to staleness. If there was
    /// no response, it returns a network error. (Can only be used when request’s mode is "same-origin". Any cached
    /// redirects will be followed assuming request’s redirect mode is "follow" and the redirects do not violate
    /// request’s mode.)
    OnlyIfCached,
}

impl From<Cache> for RequestCache {
    fn from(value: Cache) -> Self {
        match value {
            Cache::Default => RequestCache::Default,
            Cache::NoStore => RequestCache::NoStore,
            Cache::Reload => RequestCache::Reload,
            Cache::NoCache => RequestCache::NoCache,
            Cache::ForceCache => RequestCache::ForceCache,
            Cache::OnlyIfCached => RequestCache::OnlyIfCached,
        }
    }
}
