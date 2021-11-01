use std::{
    net::{SocketAddr, TcpStream},
    sync::{Arc, RwLock},
    time::Duration,
};

use crate::MockBuilder;
use rand::Rng;

/// A running gRPC server
#[derive(Clone)]
pub struct MockGrpcServer {
    pub(crate) address: SocketAddr,
    inner: Arc<Option<Inner>>,
    pub(crate) rules: Arc<RwLock<Vec<MockBuilder>>>,
}

struct Inner {
    #[allow(dead_code)]
    join_handle: tokio::task::JoinHandle<Result<(), tonic::transport::Error>>,
}

impl Drop for MockGrpcServer {
    fn drop(&mut self) {
        if let Some(r) = self.inner.as_ref() {
            println!("Terminating server");
            drop(&r.join_handle);
        }
    }
}

impl MockGrpcServer {
    pub(crate) fn new(port: u16) -> Self {
        Self {
            address: format!("[::1]:{}", port).parse().unwrap(),
            inner: Arc::default(),
            rules: Arc::default(),
        }
    }

    pub async fn start_default<F>(f: F) -> Self
    where
        F: Fn() -> tokio::task::JoinHandle<Result<(), tonic::transport::Error>>,
    {
        let port = MockGrpcServer::find_unused_port()
            .await
            .expect("Unable to find an open port");

        MockGrpcServer::new(port).start(f).await
    }

    pub(crate) async fn find_unused_port() -> Option<u16> {
        let mut rng = rand::thread_rng();

        loop {
            let port: u16 = rng.gen_range(50000..60000);
            let addr: SocketAddr = format!("[::1]:{}", port).parse().unwrap();

            if !TcpStream::connect_timeout(&addr, std::time::Duration::from_millis(25)).is_ok() {
                return Some(port);
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    }

    pub async fn start<F>(mut self, f: F) -> Self
    where
        F: Fn() -> tokio::task::JoinHandle<Result<(), tonic::transport::Error>>,
    {
        println!("Starting gRPC started in {}", self.address());

        // let thread = tokio::spawn(
        //     tonic::transport::Server::builder()
        //         .add_service(self.clone())
        //         .serve(self.address),
        // );
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
}
