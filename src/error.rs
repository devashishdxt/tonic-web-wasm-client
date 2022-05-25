use http::header::{InvalidHeaderName, InvalidHeaderValue, ToStrError};
use thiserror::Error;

/// Errors returned by `tonic-web-wasm-client`
#[derive(Debug, Error)]
pub enum ClientError {
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
    /// Invalid header name
    #[error("invalid header name")]
    InvalidHeaderName(#[from] InvalidHeaderName),
    /// Invalid header value
    #[error("invalid header value")]
    InvalidHeaderValue(#[from] InvalidHeaderValue),
    /// Missing `content-type` header in gRPC response
    #[error("missing content-type header in grpc response")]
    MissingContentTypeHeader,
    /// HTTP request failure error
    #[error("http request failed")]
    ReqwestError(#[from] reqwest::Error),
    /// gRPC error
    #[error("grpc error")]
    TonicStatusError(#[from] tonic::Status),
    /// Integer conversion error
    #[error("integer conversion error")]
    TryFromIntError(#[from] std::num::TryFromIntError),
}
