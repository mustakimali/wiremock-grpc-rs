use log::debug;
use std::{
    net::{SocketAddr, TcpStream},
    sync::{Arc, RwLock},
    task::Poll,
    time::Duration,
};
use tonic::{
    codegen::{http, Body, Never, StdError},
    Code, Status,
};

#[derive(Clone)]
pub struct MockGrpcServer {
    address: SocketAddr,
    inner: Arc<Option<Inner>>,
    rules: Arc<RwLock<Vec<RequestBuilder>>>,
}

struct Inner {
    #[allow(dead_code)]
    join_handle: tokio::task::JoinHandle<Result<(), tonic::transport::Error>>,
}

impl MockGrpcServer {
    pub fn new(port: u16) -> Self {
        Self {
            address: format!("[::1]:{}", port).parse().unwrap(),
            inner: Arc::default(),
            rules: Arc::default(),
        }
    }

    pub async fn start(mut self) -> Self {
        println!("Starting gRPC started in {}", self.address());

        let thread = tokio::spawn(
            tonic::transport::Server::builder()
                .add_service(self.clone())
                .serve(self.address),
        );

        for _ in 0..40 {
            if TcpStream::connect_timeout(&self.address, std::time::Duration::from_millis(25))
                .is_ok()
            {
                break;
            }
            debug!("WAITING...");
            tokio::time::sleep(Duration::from_millis(25)).await;
        }

        self.inner = Arc::new(Some(Inner {
            join_handle: thread,
        }));

        println!("Server started in {}", self.address());
        self
    }

    pub fn setup(&mut self, r: RequestBuilder) -> MockGrpcServer {
        r.mount(self);

        self.to_owned()
    }

    pub fn address(&self) -> &SocketAddr {
        &self.address
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

        let path = req.uri().path();
        let inner = self.rules.as_ref();
        let inner = inner.read().unwrap();
        if let Some(req_builder) = inner.iter().find(|x| x.path == path) {
            println!("Matched rule {:?}", req_builder);
            let builder = http::Response::builder()
                .status(200)
                .header("content-type", "application/grpc")
                .header(
                    "grpc-status",
                    Status::new(req_builder.status_code.unwrap_or(Code::Ok), "").to_string(),
                );

            if let Some(body) = &req_builder.result {
                let body = body.clone();
                return Box::pin(async move {
                    let s = prost::bytes::Bytes::from(body);
                    let b = http_body::Full::new(s);
                    let b = http_body::combinators::BoxBody::new(b)
                        .map_err(|err| match err {})
                        .boxed_unsync();
                    let b = tonic::body::BoxBody::new(b);
                    let b = builder.body(b).unwrap();

                    Ok(b)
                });
            } else {
                return Box::pin(async move {
                    let b = builder.body(tonic::body::empty_body()).unwrap();
                    Ok(b)
                });
            };
        }

        Box::pin(async move {
            Ok(http::Response::builder()
                .status(200)
                .header("grpc-status", "12")
                .header("content-type", "application/grpc")
                .body(tonic::body::empty_body())
                .unwrap())
        })
    }
}

#[derive(Debug)]
pub struct RequestBuilder {
    path: String,
    status_code: Option<tonic::Code>,
    result: Option<Vec<u8>>,
}

impl RequestBuilder {
    pub fn given(path: &str) -> Self {
        Self {
            path: path.into(),
            result: None,
            status_code: None,
        }
    }

    pub fn when(&self) -> Self {
        todo!()
    }

    pub fn return_status(self, status: tonic::Code) -> Self {
        Self {
            status_code: Some(status),
            ..self
        }
    }

    pub fn return_body<T, F>(self, f: F) -> Self
    where
        F: Fn() -> T,
        T: prost::Message,
    {
        let result = f();
        let result = result.encode_to_vec();

        Self {
            result: Some(result),
            ..self
        }
    }

    pub fn mount(self, s: &mut MockGrpcServer) {
        if self.status_code.is_none() && self.result.is_none() {
            panic!("Must set the status code or body before attempting to mount the rule.");
        }

        s.rules.write().unwrap().push(self);
    }
}
