use prost::Message;
use std::{sync::Arc, task::Poll};
use tonic::codegen::{http, Body, Never, StdError};

#[derive(Clone)]
pub struct MockGrpcServer {
    port: u32,
    inner: Arc<Option<Inner>>,
}

struct Inner {
    #[allow(dead_code)]
    join_handle: tokio::task::JoinHandle<Result<(), tonic::transport::Error>>,
    rules: Vec<RequestBuilder>,
}

impl MockGrpcServer {
    pub fn new(port: u32) -> Self {
        Self {
            port,
            inner: Arc::default(),
        }
    }

    pub fn start(mut self) -> Self {
        let thread = tokio::spawn(
            tonic::transport::Server::builder()
                .add_service(self.clone())
                .serve(format!("[::1]:{}", self.port).parse().unwrap()),
        );

        self.inner = Arc::new(Some(Inner {
            join_handle: thread,
            rules: Vec::default(),
        }));
        self
    }

    pub fn setup<T: prost::Message>(self, r: RequestBuilder) -> MockGrpcServer {
        r.mount(&self);

        self
    }
}

impl tonic::transport::NamedService for MockGrpcServer {
    const NAME: &'static str = "wiremock.server";
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

pub struct RequestBuilder {
    path: String,
    status_code: Option<tonic::Status>,
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

    pub fn return_status(self, status: tonic::Status) -> Self {
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

    pub fn mount(self, s: &MockGrpcServer) {
        if self.status_code.is_none() && self.result.is_none() {
            panic!("Must set the status code or body before attempting to mount the rule.");
        }

        //s.inner.unwrap().rules.push(Box::new(self));
    }
}
