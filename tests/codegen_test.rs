use std::{
    ops::{Deref, DerefMut},
    task::Poll,
};
use tonic::codegen::http;
use wiremock_grpc::*;

#[tokio::test]
async fn codegen_works() {
    let server = Server::start_default().await;

    assert!(std::net::TcpStream::connect(&server.address()).is_ok())
}

//
// Sample generated code
//

//---------------
// GEN CODES    |
//---------------
#[derive(Clone)]
pub struct Server(pub(crate) GrpcServer);

impl Deref for Server {
    type Target = GrpcServer;

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
    B: http_body::Body + Send + 'static,
    B::Error: Into<tonic::codegen::StdError> + Send + 'static,
{
    type Response = tonic::codegen::http::Response<tonic::body::BoxBody>;
    type Error = std::convert::Infallible;
    type Future = tonic::codegen::BoxFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: http::Request<B>) -> Self::Future {
        self.0.handle_request(req)
    }
}

impl tonic::server::NamedService for Server {
    const NAME: &'static str = "hello.Greeter";
}

impl Server {
    pub async fn start_default() -> Self {
        let port = GrpcServer::find_unused_port()
            .await
            .expect("Unable to find an open port");

        Self(GrpcServer::new(port)).start_internal().await
    }

    pub async fn start(port: u16) -> Self {
        Self(GrpcServer::new(port)).start_internal().await
    }

    async fn start_internal(&mut self) -> Self {
        let address = self.address().clone();
        let thread = tokio::spawn(
            tonic::transport::Server::builder()
                .add_service(self.clone())
                .serve(address),
        );
        self._start(thread).await;
        self.to_owned()
    }
}

//---------------
// GEN CODES    |
//---------------
