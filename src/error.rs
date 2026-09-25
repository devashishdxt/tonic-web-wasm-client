use http::header::{InvalidHeaderName, InvalidHeaderValue, ToStrError};
use js_sys::JsString;
use thiserror::Error;
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

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
    #[error("HTTP error")]
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
    #[error("JS API error: {0}")]
    JsError(String),
    /// Malformed response
    #[error("malformed response")]
    MalformedResponse,
    /// Missing `content-type` header in gRPC response
    #[error("missing content-type header in gRPC response")]
    MissingContentTypeHeader,
    /// Missing response body in HTTP call
    #[error("missing response body in HTTP call")]
    MissingResponseBody,
    /// gRPC error
    #[error("gRPC error")]
    TonicStatusError(#[from] tonic::Status),
}

impl Error {
    /// Initialize js error from js value
    pub(crate) fn js_error(value: JsValue) -> Self {
        let message = js_object_display(&value);

        if message.contains("tonic_web_wasm_client::Error::TimedOut") {
            Self::TonicStatusError(tonic::Status::deadline_exceeded("Request timed out"))
        } else {
            Self::JsError(message)
        }
    }
}

#[wasm_bindgen]
extern "C" {
    // `String(value)` describes any value, including `null`, `undefined` and objects without a
    // `toString` method. It is marked `catch` because it throws when `toString` throws.
    #[wasm_bindgen(js_name = String, catch)]
    fn js_string(value: &JsValue) -> Result<JsString, JsValue>;
}

fn js_object_display(value: &JsValue) -> String {
    match js_string(value) {
        Ok(string) => string.into(),
        Err(_) => format!("{value:?}"),
    }
}
