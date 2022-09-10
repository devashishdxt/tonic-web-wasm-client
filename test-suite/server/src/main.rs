use std::{
    error::Error,
    pin::Pin,
    task::{Context, Poll},
};

use futures_core::Stream;
use proto::echo_server::EchoServer;
use tonic::{transport::Server, Request, Response, Status};

use self::proto::{echo_server::Echo, EchoRequest, EchoResponse};

pub mod proto {
    tonic::include_proto!("echo");
}

pub struct EchoService;

#[tonic::async_trait]
impl Echo for EchoService {
    type EchoStreamStream = MessageStream;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = "[::1]:50051".parse().unwrap();
    let echo = EchoServer::new(EchoService);

    Server::builder()
        .accept_http1(true)
        .add_service(tonic_web::enable(echo))
        .serve(addr)
        .await?;

    Ok(())
}
