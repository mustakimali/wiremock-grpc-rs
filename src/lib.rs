use std::task::Poll;

use tonic::codegen::{http, Body, Never, StdError};

#[derive(Clone)]
struct MockGrpcServer {
    port: u32,
    inner: Inner,
}

#[derive(Clone, Default)]
struct Inner {
    //join_handle: Option<JoinHandle<>>,
}

mod greeter_code;

impl MockGrpcServer {
    pub fn new(port: u32) -> Self {
        Self {
            port,
            inner: Inner::default(),
        }
    }

    pub fn start(self) -> Self {
        // let _ = tokio::spawn(
        //     tonic::transport::Server::builder()
        //         .add_service(self.clone())
        //         .serve(format!("[::1]:{}", self.port).parse().unwrap()),
        // );

        self
    }
}

impl tonic::transport::NamedService for MockGrpcServer {
    const NAME: &'static str = "hello.Greeter";
}

impl<B> tonic::codegen::Service<http::Request<B>> for MockGrpcServer
where
    B: Body + Send + 'static,
    B::Error: Into<StdError> + Send + 'static,
{
    type Response = http::Response<tonic::body::BoxBody>;
    type Error = Never;
    type Future = tonic::codegen::BoxFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: http::Request<B>) -> Self::Future {
        println!("Request to {}", req.uri().path());

        match req.uri().path() {
            _ => Box::pin(async move {
                Ok(http::Response::builder()
                    .status(200)
                    .header("grpc-status", "12")
                    .header("content-type", "application/grpc")
                    .body(tonic::body::empty_body())
                    .unwrap())
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::{
        greeter_code::{greeter_client, HelloRequest},
        MockGrpcServer,
    };

    #[tokio::test]
    async fn it_works() {
        let svr = MockGrpcServer::new(50055).start();
        let _ = tokio::spawn(
            tonic::transport::Server::builder()
                .add_service(svr.clone())
                .serve(format!("[::1]:{}", &svr.port).parse().unwrap()),
        );

        tokio::time::sleep(Duration::from_secs(2)).await;

        let channel = tonic::transport::Channel::from_static("http://[::1]:50055")
            .connect()
            .await
            .unwrap();
        let mut client = greeter_client::GreeterClient::new(channel);
        let _r = client
            .say_hello(HelloRequest {
                name: "Yo yo".into(),
            })
            .await
            .unwrap();
    }

    #[test]
    fn create() {
        let cd = std::env::current_dir().unwrap();
        std::env::set_var("OUT_DIR", &cd);
        let cd = cd.join("hello.proto");
        tonic_build::compile_protos(cd).expect("Unable to generate the code");
    }
}
