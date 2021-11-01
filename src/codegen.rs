#[macro_export]
macro_rules! generate {
    ($prefix:literal, $type: ident) => {
        use std::{
            ops::{Deref, DerefMut},
            task::Poll,
        };
        use tonic::{
            codegen::{http, Body, StdError},
            Code,
        };

        use wiremock_grpc::*;

        #[derive(Clone)]
        pub struct $type(pub(crate) MockGrpcServer);

        impl Deref for $type {
            type Target = MockGrpcServer;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl DerefMut for $type {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl<B> tonic::codegen::Service<tonic::codegen::http::Request<B>> for $type
        where
            B: Body + Send + 'static,
            B::Error: Into<StdError> + Send + 'static,
        {
            type Response = tonic::codegen::http::Response<tonic::body::BoxBody>;
            type Error = tonic::codegen::Never;
            type Future = tonic::codegen::BoxFuture<Self::Response, Self::Error>;

            fn poll_ready(
                &mut self,
                _cx: &mut std::task::Context<'_>,
            ) -> Poll<Result<(), Self::Error>> {
                Poll::Ready(Ok(()))
            }

            fn call(&mut self, req: http::Request<B>) -> Self::Future {
                self.0.handle_request(req)
            }
        }

        impl tonic::transport::NamedService for $type {
            const NAME: &'static str = $prefix;
        }

        impl $type {
            pub async fn start_default() -> Self {
                let port = MockGrpcServer::find_unused_port()
                    .await
                    .expect("Unable to find an open port");

                Self(MockGrpcServer::new(port)).start_internal(port).await
            }

            pub async fn start(port: u16) -> Self {
                Self(MockGrpcServer::new(port)).start_internal(port).await
            }

            async fn start_internal(&self, port: u16) -> Self {
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
                self.to_owned()
            }
        }
    };
}
