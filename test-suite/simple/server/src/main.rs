use std::{
    error::Error,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use futures_core::Stream;
use http::header::HeaderName;
use proto::echo_server::EchoServer;
use tonic::{transport::Server, Request, Response, Status};
use tonic_web::GrpcWebLayer;
use tower::{Layer, Service};
use tower_http::cors::{AllowOrigin, CorsLayer};

use self::proto::{echo_server::Echo, EchoRequest, EchoResponse};

pub mod proto {
    tonic::include_proto!("echo");
}

pub struct EchoService;

#[tonic::async_trait]
impl Echo for EchoService {
    type EchoStreamStream = MessageStream;

    type EchoInfiniteStreamStream = InfiniteMessageStream;

    async fn echo(&self, request: Request<EchoRequest>) -> Result<Response<EchoResponse>, Status> {
        let request = request.into_inner();
        Ok(Response::new(EchoResponse {
            message: format!("echo({})", request.message),
        }))
    }

    async fn echo_timeout(
        &self,
        request: Request<EchoRequest>,
    ) -> Result<Response<EchoResponse>, Status> {
        let request = request.into_inner();
        // Simulate a long processing time to trigger client timeout
        tokio::time::sleep(Duration::from_secs(10)).await;
        Ok(Response::new(EchoResponse {
            message: format!("echo({})", request.message),
        }))
    }

    async fn echo_stream(
        &self,
        request: Request<EchoRequest>,
    ) -> Result<Response<Self::EchoStreamStream>, Status> {
        let request = request.into_inner();
        Ok(Response::new(MessageStream::new(request.message)))
    }

    async fn echo_infinite_stream(
        &self,
        request: tonic::Request<EchoRequest>,
    ) -> Result<tonic::Response<Self::EchoInfiniteStreamStream>, tonic::Status> {
        let request = request.into_inner();
        Ok(Response::new(InfiniteMessageStream::new(request.message)))
    }

    type EchoStreamErrorStream = ErrorAfterMessagesStream;

    async fn echo_stream_error(
        &self,
        request: Request<EchoRequest>,
    ) -> Result<Response<Self::EchoStreamErrorStream>, Status> {
        let request = request.into_inner();
        Ok(Response::new(ErrorAfterMessagesStream::new(request.message)))
    }

    async fn echo_error_response(
        &self,
        _: tonic::Request<EchoRequest>,
    ) -> Result<Response<EchoResponse>, tonic::Status> {
        Err(tonic::Status::unauthenticated("user not authenticated"))
    }
}

pub struct MessageStream {
    message: String,
    count: u8,
}

impl MessageStream {
    pub fn new(message: String) -> Self {
        Self { message, count: 0 }
    }
}

impl Stream for MessageStream {
    type Item = Result<EchoResponse, Status>;

    fn poll_next(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.count < 3 {
            self.count += 1;
            Poll::Ready(Some(Ok(EchoResponse {
                message: format!("echo({})", self.message),
            })))
        } else {
            Poll::Ready(None)
        }
    }
}

pub struct InfiniteMessageStream {
    message: String,
    count: u8,
}

impl InfiniteMessageStream {
    pub fn new(message: String) -> Self {
        Self { message, count: 0 }
    }
}

impl Stream for InfiniteMessageStream {
    type Item = Result<EchoResponse, Status>;

    fn poll_next(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.count = self.count.saturating_add(1);

        Poll::Ready(Some(Ok(EchoResponse {
            message: format!("echo({}, {})", self.message, self.count),
        })))
    }
}

pub struct ErrorAfterMessagesStream {
    message: String,
    count: u8,
}

impl ErrorAfterMessagesStream {
    pub fn new(message: String) -> Self {
        Self { message, count: 0 }
    }
}

impl Stream for ErrorAfterMessagesStream {
    type Item = Result<EchoResponse, Status>;

    fn poll_next(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.count < 2 {
            self.count += 1;
            Poll::Ready(Some(Ok(EchoResponse {
                message: format!("echo({})", self.message),
            })))
        } else {
            Poll::Ready(Some(Err(Status::internal("stream error after 2 messages"))))
        }
    }
}

/// Answers requests under `/status/<code>/` with that HTTP status and a plain text body, the way
/// a proxy or load balancer error page looks, so tests can check how the client handles
/// responses that do not come from a gRPC server.
#[derive(Clone)]
struct HttpStatusLayer;

impl<S> Layer<S> for HttpStatusLayer {
    type Service = HttpStatusService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        HttpStatusService { inner }
    }
}

#[derive(Clone)]
struct HttpStatusService<S> {
    inner: S,
}

impl<S, B> Service<http::Request<B>> for HttpStatusService<S>
where
    S: Service<http::Request<B>, Response = http::Response<tonic::body::Body>>,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: http::Request<B>) -> Self::Future {
        let status = request
            .uri()
            .path()
            .strip_prefix("/status/")
            .and_then(|rest| rest.split('/').next())
            .and_then(|code| code.parse::<http::StatusCode>().ok());

        match status {
            Some(status) => Box::pin(async move {
                Ok(http::Response::builder()
                    .status(status)
                    .header(http::header::CONTENT_TYPE, "text/plain")
                    .body(tonic::body::Body::new(
                        "upstream connect error or disconnect/reset before headers".to_string(),
                    ))
                    .unwrap())
            }),
            None => Box::pin(self.inner.call(request)),
        }
    }
}

const DEFAULT_MAX_AGE: Duration = Duration::from_secs(24 * 60 * 60);
const DEFAULT_EXPOSED_HEADERS: [HeaderName; 3] = [
    HeaderName::from_static("grpc-status"),
    HeaderName::from_static("grpc-message"),
    HeaderName::from_static("grpc-status-details-bin"),
];
const DEFAULT_ALLOW_HEADERS: [HeaderName; 4] = [
    HeaderName::from_static("x-grpc-web"),
    HeaderName::from_static("content-type"),
    HeaderName::from_static("x-user-agent"),
    HeaderName::from_static("grpc-timeout"),
];

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = "[::1]:50051".parse().unwrap();
    let echo = EchoServer::new(EchoService);

    Server::builder()
        .accept_http1(true)
        .layer(
            CorsLayer::new()
                .allow_origin(AllowOrigin::mirror_request())
                .allow_credentials(true)
                .max_age(DEFAULT_MAX_AGE)
                .expose_headers(DEFAULT_EXPOSED_HEADERS)
                .allow_headers(DEFAULT_ALLOW_HEADERS),
        )
        .layer(HttpStatusLayer)
        .layer(GrpcWebLayer::new())
        .add_service(echo)
        .serve(addr)
        .await?;

    Ok(())
}
