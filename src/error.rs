use http::header::ToStrError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("failed to convert header value to string")]
    HeaderValueError(#[from] ToStrError),
    #[error("http error")]
    HttpError(#[from] http::Error),
    #[error("http request failed")]
    ReqwestError(#[from] reqwest::Error),
    #[error("grpc error")]
    TonicStatusError(#[from] tonic::Status),
}
