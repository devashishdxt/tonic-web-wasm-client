use std::{future::Future, pin::Pin};

use bytes::Bytes;
use http::{
    header::{ACCEPT, CONTENT_TYPE},
    Request, Response,
};
use http_body::{Body, Full};
use tonic::body::BoxBody;
use tower::Service;

use crate::{error::ClientError, utils::set_panic_hook};

#[derive(Debug, Clone)]
pub struct Client {
    base_url: String,
}

impl Client {
    pub fn new(base_url: String) -> Self {
        set_panic_hook();
        Self { base_url }
    }
}

impl Service<Request<BoxBody>> for Client {
    type Response = Response<Full<Bytes>>;

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
) -> Result<Response<Full<Bytes>>, ClientError> {
    url.push_str(&req.uri().to_string());

    let client = reqwest::Client::new();
    let mut builder = client.post(&url);

    builder = builder
        .header(CONTENT_TYPE, "application/grpc-web+proto")
        .header(ACCEPT, "application/grpc-web+proto");

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

    let body = Full::new(response.bytes().await?);

    result.body(body).map_err(Into::into)
}
