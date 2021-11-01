use std::{
    net::{SocketAddr, TcpStream},
    sync::{Arc, RwLock},
    time::Duration,
};

use crate::tonic_ext::{GenericCodec, SvcGeneric};
use crate::MockBuilder;
use rand::Rng;
use tonic::{
    codegen::{
        http::{self, HeaderValue},
        Body, Never, StdError,
    },
    Code,
};

/// A running gRPC server
#[derive(Clone)]
pub struct MockGrpcServer {
    pub(crate) address: SocketAddr,
    inner: Arc<Option<Inner>>,
    pub(crate) rules: Arc<RwLock<Vec<RwLock<RuleItem>>>>,
}

#[derive(Debug)]
pub(crate) struct RuleItem {
    pub(crate) match_count: u32,
    pub(crate) rule: MockBuilder,
}

impl RuleItem {
    fn record_request(&mut self) {
        self.match_count += 1;
    }
}

struct Inner {
    #[allow(dead_code)]
    join_handle: tokio::task::JoinHandle<Result<(), tonic::transport::Error>>,
}

impl Drop for MockGrpcServer {
    fn drop(&mut self) {
        if self.inner.as_ref().is_some() {
            println!("Terminating server");
        }
    }
}

impl MockGrpcServer {
    pub fn new(port: u16) -> Self {
        Self {
            address: format!("[::1]:{}", port).parse().unwrap(),
            inner: Arc::default(),
            rules: Arc::default(),
        }
    }

    pub async fn _start_default<F>(f: F) -> Self
    where
        F: Fn() -> tokio::task::JoinHandle<Result<(), tonic::transport::Error>>,
    {
        let port = MockGrpcServer::find_unused_port()
            .await
            .expect("Unable to find an open port");

        MockGrpcServer::new(port)._start(f).await
    }

    pub async fn find_unused_port() -> Option<u16> {
        let mut rng = rand::thread_rng();

        loop {
            let port: u16 = rng.gen_range(50000..60000);
            let addr: SocketAddr = format!("[::1]:{}", port).parse().unwrap();

            if TcpStream::connect_timeout(&addr, std::time::Duration::from_millis(25)).is_err() {
                return Some(port);
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    }

    pub async fn _start<F>(mut self, f: F) -> Self
    where
        F: Fn() -> tokio::task::JoinHandle<Result<(), tonic::transport::Error>>,
    {
        println!("Starting gRPC started in {}", self.address());

        let thread = f();

        for _ in 0..40 {
            if TcpStream::connect_timeout(&self.address, std::time::Duration::from_millis(25))
                .is_ok()
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }

        self.inner = Arc::new(Some(Inner {
            join_handle: thread,
        }));

        println!("Server started in {}", self.address());
        self
    }

    pub fn setup<M>(&mut self, r: M) -> MockGrpcServer
    where
        M: crate::Then + crate::Mountable,
    {
        r.mount(self);

        self.to_owned()
    }

    pub fn address(&self) -> &SocketAddr {
        &self.address
    }

    pub fn handle_request<B>(
        &self,
        req: http::Request<B>,
    ) -> tonic::codegen::BoxFuture<http::Response<tonic::body::BoxBody>, Never>
    where
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        println!("Request to {}", req.uri().path());

        let path = req.uri().path();
        let inner = self.rules.as_ref();
        let inner = inner.read().unwrap();

        if let Some(item) = inner.iter().find(|x| x.read().unwrap().rule.path == path) {
            println!("Matched rule {:?}", item);
            item.write().unwrap().record_request();

            let item = item.read().unwrap();
            let code = item.rule.status_code.unwrap_or(Code::Ok);
            if let Some(body) = &item.rule.result {
                println!("Returning body ({} bytes)", body.len());
                let body = body.clone();

                let fut = async move {
                    let method = SvcGeneric(body);
                    let codec = GenericCodec::default();

                    let mut grpc = tonic::server::Grpc::new(codec);
                    let mut result = grpc.unary(method, req).await;
                    result.headers_mut().append(
                        "grpc-status",
                        HeaderValue::from_str(format!("{}", code as u32).as_str()).unwrap(),
                    );
                    Ok(result)
                };
                return Box::pin(fut);
            } else {
                let status = code as u32;
                let builder = http::Response::builder()
                    .status(200)
                    .header("content-type", "application/grpc")
                    .header("grpc-status", format!("{}", status));
                println!("Returning empty body with status {}", status);

                return Box::pin(async move {
                    let body = builder.body(tonic::body::empty_body()).unwrap();
                    Ok(body)
                });
            };
        }

        println!("Request unhandled");
        panic!("Mock is not setup for {}", path);
    }
}
