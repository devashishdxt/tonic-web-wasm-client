use http::header::{InvalidHeaderName, InvalidHeaderValue, ToStrError};
use thiserror::Error;

/// Error type for `tonic-web-wasm-client`
#[derive(Debug, Error)]
pub enum Error {
    /// Base64 decode error
    #[error("base64 decode error")]
    Base64DecodeError(#[from] base64::DecodeError),
    /// Header parsing error
    #[error("failed to parse headers")]
    HeaderParsingError,
    /// Header value error
    #[error("failed to convert header value to string")]
    HeaderValueError(#[from] ToStrError),
    /// HTTP error
    #[error("http error")]
    HttpError(#[from] http::Error),
    /// Invalid content type
    #[error("invalid content type: {0}")]
    InvalidContentType(String),
    /// Invalid header name
    #[error("invalid header name")]
    InvalidHeaderName(#[from] InvalidHeaderName),
    /// Invalid header value
    #[error("invalid header value")]
    InvalidHeaderValue(#[from] InvalidHeaderValue),
    /// JS API error
    #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
    #[error("js api error: {0}")]
    JsError(String),
    /// Malformed response
    #[error("malformed response")]
    MalformedResponse,
    /// Missing `content-type` header in gRPC response
    #[error("missing content-type header in grpc response")]
    MissingContentTypeHeader,
    /// Missing response body in HTTP call
    #[error("missing response body in HTTP call")]
    MissingResponseBody,
    /// gRPC error
    #[error("grpc error")]
    TonicStatusError(#[from] tonic::Status),
    #[cfg(all(target_arch = "wasm32", target_os = "wasi", target_env = "p2"))]
    #[error(transparent)]
    WstdHttp(wstd::http::Error),
}

impl Error {
    /// Initialize js error from js value
    #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
    pub(crate) fn js_error(value: wasm_bindgen::JsValue) -> Self {
        let message = js_object_display(&value);
        Self::JsError(message)
    }
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
fn js_object_display(option: &wasm_bindgen::JsValue) -> String {
    use wasm_bindgen::JsCast;
    let object: &js_sys::Object = option.unchecked_ref();
    ToString::to_string(&object.to_string())
}
