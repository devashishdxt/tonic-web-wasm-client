use std::error::Error;

use proto::echo_server::EchoServer;
use tonic::{transport::Server, Request, Response, Status};

use self::proto::{echo_server::Echo, EchoRequest, EchoResponse};

pub mod proto {
    tonic::include_proto!("echo");
}

pub struct EchoService;

#[tonic::async_trait]
impl Echo for EchoService {
    async fn echo(&self, request: Request<EchoRequest>) -> Result<Response<EchoResponse>, Status> {
        let request = request.into_inner();
        Ok(Response::new(EchoResponse {
            message: format!("echo({})", request.message),
        }))
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
