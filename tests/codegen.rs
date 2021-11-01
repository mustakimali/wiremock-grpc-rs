use std::{
    ops::{Deref, DerefMut},
    task::Poll,
};
use tonic::{
    codegen::{http, Body, StdError},
    Code,
};

use wiremock_grpc::*;

#[tokio::test]
async fn codegen_works() {
    let mut server = Server::start_default().await;

    server.setup(
        MockBuilder::when()
            .path("")
            .then()
            .return_status(Code::Aborted),
    );

    assert!(std::net::TcpStream::connect(&server.address()).is_ok())
}

//
// Sample generated code
//

//---------------
// GEN CODES    |
//---------------
#[derive(Clone)]
pub struct Server(pub(crate) MockGrpcServer);

impl Deref for Server {
    type Target = MockGrpcServer;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Server {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<B> tonic::codegen::Service<tonic::codegen::http::Request<B>> for Server
where
    B: Body + Send + 'static,
    B::Error: Into<StdError> + Send + 'static,
{
    type Response = tonic::codegen::http::Response<tonic::body::BoxBody>;
    type Error = tonic::codegen::Never;
    type Future = tonic::codegen::BoxFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: http::Request<B>) -> Self::Future {
        self.0.handle_request(req)
    }
}

impl tonic::transport::NamedService for Server {
    const NAME: &'static str = "hello.Greeter";
}

impl Server {
    pub async fn start_default() -> Self {
        let port = MockGrpcServer::find_unused_port()
            .await
            .expect("Unable to find an open port");

        Self(MockGrpcServer::new(port)).start(port).await
    }

    pub async fn start(&self, port: u16) -> Self {
        let grpc_serve = MockGrpcServer::new(port);
        let address = grpc_serve.address().clone();
        let grpc_server = grpc_serve
            ._start(|| {
                tokio::spawn(
                    tonic::transport::Server::builder()
                        .add_service(self.clone())
                        .serve(address),
                )
            })
            .await;
        Self(grpc_server)
    }
}

//---------------
// GEN CODES    |
//---------------
