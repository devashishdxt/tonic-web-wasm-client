use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::Future;
use http::{Request, Response};
use tonic::body::BoxBody;
use tower_service::Service;

use crate::{
    call::{call, FetchOptions},
    Error, ResponseBody,
};

/// `grpc-web` based transport layer for `tonic` clients
#[derive(Debug, Clone)]
pub struct Client {
    base_url: String,
    options: FetchOptions,
}

impl Client {
    /// Creates a new client
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            options: FetchOptions::default(),
        }
    }

    /// Creates a new client with custom options
    pub fn new_with_options(base_url: String, options: FetchOptions) -> Self {
        Self { base_url, options }
    }
}

impl Service<Request<BoxBody>> for Client {
    type Response = Response<ResponseBody>;

    type Error = Error;

    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: Request<BoxBody>) -> Self::Future {
        Box::pin(call(self.base_url.clone(), request, self.options.clone()))
    }
}
