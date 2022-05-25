use http::header::{InvalidHeaderName, InvalidHeaderValue, ToStrError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("base64 decode error")]
    Base64DecodeError(#[from] base64::DecodeError),
    #[error("failed to parse headers")]
    HeaderParsingError,
    #[error("failed to convert header value to string")]
    HeaderValueError(#[from] ToStrError),
    #[error("http error")]
    HttpError(#[from] http::Error),
    #[error("invalid header name")]
    InvalidHeaderName(#[from] InvalidHeaderName),
    #[error("invalid header value")]
    InvalidHeaderValue(#[from] InvalidHeaderValue),
    #[error("missing content-type header in grpc response")]
    MissingContentTypeHeader,
    #[error("http request failed")]
    ReqwestError(#[from] reqwest::Error),
    #[error("grpc error")]
    TonicStatusError(#[from] tonic::Status),
    #[error("integer conversion error")]
    TryFromIntError(#[from] std::num::TryFromIntError),
}
