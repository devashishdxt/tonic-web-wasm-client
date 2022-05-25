use std::{future::Future, pin::Pin};

use bytes::{Bytes, BytesMut};
use http::{
    header::{ACCEPT, CONTENT_TYPE},
    Request, Response,
};
use http_body::{combinators::UnsyncBoxBody, Body};
use tonic::body::BoxBody;
use tower::Service;

use crate::{error::ClientError, grpc_response::GrpcResponse, utils::set_panic_hook};

/// `grpc-web` based transport layer for `tonic` clients
#[derive(Debug, Clone)]
pub struct Client {
    base_url: String,
}

impl Client {
    /// Creates a new client
    pub fn new(base_url: String) -> Self {
        set_panic_hook();
        Self { base_url }
    }
}

impl Service<Request<BoxBody>> for Client {
    type Response = Response<UnsyncBoxBody<Bytes, ClientError>>;

    type Error = ClientError;

    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Ok(()).into()
    }

    fn call(&mut self, req: Request<BoxBody>) -> Self::Future {
        Box::pin(request(self.base_url.clone(), req))
    }
}

async fn request(
    mut url: String,
    req: Request<BoxBody>,
) -> Result<Response<UnsyncBoxBody<Bytes, ClientError>>, ClientError> {
    url.push_str(&req.uri().to_string());

    let client = reqwest::Client::new();
    let mut builder = client.post(&url);

    builder = builder
        .header(CONTENT_TYPE, "application/grpc-web+proto")
        .header(ACCEPT, "application/grpc-web+proto")
        .header("x-grpc-web", "1")
        .fetch_credentials_same_origin();

    for (header_name, header_value) in req.headers().iter() {
        builder = builder.header(header_name.as_str(), header_value.to_str()?);
    }

    let body = req.into_body().data().await;
    if let Some(body) = body {
        builder = builder.body(body?);
    }

    let response = builder.send().await?;

    let mut result = Response::builder();
    result = result.status(response.status());

    for (header_name, header_value) in response.headers().iter() {
        result = result.header(header_name.as_str(), header_value.to_str()?);
    }

    let content_type = match response.headers().get(CONTENT_TYPE) {
        None => Err(ClientError::MissingContentTypeHeader),
        Some(content_type) => content_type.to_str().map_err(Into::into),
    }?
    .to_owned();

    let bytes = BytesMut::from(response.bytes().await?.as_ref());
    let body = UnsyncBoxBody::new(GrpcResponse::new(bytes, &content_type)?);

    result.body(body).map_err(Into::into)
}
