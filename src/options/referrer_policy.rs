use web_sys::ReferrerPolicy as RequestReferrerPolicy;

/// Request's referrer policy
#[derive(Debug, Clone, Copy, Default)]
pub enum ReferrerPolicy {
    /// Corresponds to no referrer policy, causing a fallback to a referrer policy defined elsewhere, or in the case
    /// where no such higher-level policy is available, falling back to the default referrer policy
    None,
    /// Specifies that no referrer information is to be sent along with requests to any origin
    NoReferrer,
    /// The "no-referrer-when-downgrade" policy sends a request’s full referrerURL stripped for use as a referrer for
    /// requests:
    ///
    /// - whose referrerURL and current URL are both potentially trustworthy URLs, or
    /// - whose referrerURL is a non-potentially trustworthy URL
    NoReferrerWhenDowngrade,
    /// Specifies that only the ASCII serialization of the request’s referrerURL is sent as referrer information when
    /// making both same-origin-referrer requests and cross-origin-referrer requests
    Origin,
    /// Specifies that a request’s full referrerURL is sent as referrer information when making same-origin-referrer
    /// requests, and only the ASCII serialization of the origin of the request’s referrerURL is sent as referrer
    /// information when making cross-origin-referrer requests
    OriginWhenCrossOrigin,
    /// specifies that a request’s full referrerURL is sent along for both same-origin-referrer requests and
    /// cross-origin-referrer requests
    UnsafeUrl,
    /// Specifies that a request’s full referrerURL is sent as referrer information when making same-origin-referrer
    /// requests
    SameOrigin,
    /// The "strict-origin" policy sends the ASCII serialization of the origin of the referrerURL for requests:
    ///
    /// - whose referrerURL and current URL are both potentially trustworthy URLs, or
    /// - whose referrerURL is a non-potentially trustworthy URL.
    StrictOrigin,
    /// Specifies that a request’s full referrerURL is sent as referrer information when making same-origin-referrer
    /// requests, and only the ASCII serialization of the origin of the request’s referrerURL when making
    /// cross-origin-referrer requests:
    ///
    /// - whose referrerURL and current URL are both potentially trustworthy URLs, or
    /// - whose referrerURL is a non-potentially trustworthy URL
    #[default]
    StrictOriginWhenCrossOrigin,
}

impl From<ReferrerPolicy> for RequestReferrerPolicy {
    fn from(value: ReferrerPolicy) -> Self {
        match value {
            ReferrerPolicy::None => RequestReferrerPolicy::None,
            ReferrerPolicy::NoReferrer => RequestReferrerPolicy::NoReferrer,
            ReferrerPolicy::NoReferrerWhenDowngrade => {
                RequestReferrerPolicy::NoReferrerWhenDowngrade
            }
            ReferrerPolicy::Origin => RequestReferrerPolicy::Origin,
            ReferrerPolicy::OriginWhenCrossOrigin => RequestReferrerPolicy::OriginWhenCrossOrigin,
            ReferrerPolicy::UnsafeUrl => RequestReferrerPolicy::UnsafeUrl,
            ReferrerPolicy::SameOrigin => RequestReferrerPolicy::SameOrigin,
            ReferrerPolicy::StrictOrigin => RequestReferrerPolicy::StrictOrigin,
            ReferrerPolicy::StrictOriginWhenCrossOrigin => {
                RequestReferrerPolicy::StrictOriginWhenCrossOrigin
            }
        }
    }
}
