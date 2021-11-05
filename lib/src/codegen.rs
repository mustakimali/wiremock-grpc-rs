#[doc = include_str!("../README.md")]
#[macro_export]
macro_rules! generate {
    ($prefix:literal, $type: ident) => {
        use ::wiremock_grpc::tonic::{
            codegen::{http, Body, StdError},
            Code,
        };
        use std::{
            ops::{Deref, DerefMut},
            task::Poll,
        };

        use wiremock_grpc::*;

        /// A running gRPC server that binds to service with prefix: `
        #[doc = $prefix]
        /// `
        /// # Example
        /// ```no_run
        /// let mut server =
        #[doc = stringify!($type)]
        /// ::start_default().await;
        /// ```
        /// More documentations in [`crate`]
        #[derive(Clone)]
        pub struct $type(pub(crate) GrpcServer);

        impl Deref for $type {
            type Target = GrpcServer;

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
            B: ::wiremock_grpc::http_body::Body + Send + 'static,
            B::Error: Into<tonic::codegen::StdError> + Send + 'static,
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

            fn call(&mut self, req: tonic::codegen::http::Request<B>) -> Self::Future {
                self.0.handle_request(req)
            }
        }

        impl tonic::transport::NamedService for $type {
            const NAME: &'static str = $prefix;
        }

        impl $type {
            /// Start the server and listens to an available port.
            ///
            /// The port can be accesed using `address()`
            /// ```no_run
            /// let server = MyMockServer::start_default();
            /// let address = server.address();
            /// let port : u16 = address.port();
            /// ```
            pub async fn start_default() -> Self {
                let port = GrpcServer::find_unused_port()
                    .await
                    .expect("Unable to find an open port");

                Self(GrpcServer::new(port)).start_internal().await
            }

            /// Start the server with a specified port.
            ///
            /// ## Panics
            /// * When the the port is not available.
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
    };
}
