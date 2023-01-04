use std::{
    error::Error,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use futures_core::Stream;
use http::header::HeaderName;
use proto::echo_server::EchoServer;
use tonic::{codegen::CompressionEncoding, transport::Server, Request, Response, Status};
use tonic_web::GrpcWebLayer;
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

const DEFAULT_MAX_AGE: Duration = Duration::from_secs(24 * 60 * 60);
const DEFAULT_EXPOSED_HEADERS: [&str; 4] = [
    "grpc-status",
    "grpc-message",
    "grpc-status-details-bin",
    "grpc-encoding",
];
const DEFAULT_ALLOW_HEADERS: [&str; 5] = [
    "x-grpc-web",
    "content-type",
    "x-user-agent",
    "grpc-timeout",
    "grpc-accept-encoding",
];

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = "[::1]:50051".parse().unwrap();
    let echo = EchoServer::new(EchoService).send_compressed(CompressionEncoding::Gzip);

    Server::builder()
        .accept_http1(true)
        .layer(
            CorsLayer::new()
                .allow_origin(AllowOrigin::mirror_request())
                .allow_credentials(true)
                .max_age(DEFAULT_MAX_AGE)
                .expose_headers(
                    DEFAULT_EXPOSED_HEADERS
                        .iter()
                        .cloned()
                        .map(HeaderName::from_static)
                        .collect::<Vec<HeaderName>>(),
                )
                .allow_headers(
                    DEFAULT_ALLOW_HEADERS
                        .iter()
                        .cloned()
                        .map(HeaderName::from_static)
                        .collect::<Vec<HeaderName>>(),
                ),
        )
        .layer(GrpcWebLayer::new())
        .add_service(echo)
        .serve(addr)
        .await?;

    Ok(())
}
