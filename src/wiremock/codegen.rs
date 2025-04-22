/// Generate mock server code for the given `prefix` and `type`.
///
/// For each gRPC server you need to generate codes using this macro.
///
/// # Arguments
/// * `prefix` - The prefix of the RPC (eg. `hello.Greeter` if the RPC is `/helloworld.Greeter/SayHello`)
/// * `type` - Type of the generated server. This [`Deref`](core::ops::Deref) to [`GrpcServer`](crate::grpc_server::GrpcServer). You will be interacting with this type in your test.
///
/// # Example
/// ```no_run
/// mod wiremock_gen {
///     // hello.Greeter: is the prefix of all rpc,
///     // MyMockServer: name of the generated Server,
///     wiremock_grpc::generate!("hello.Greeter", MyMockServer);
/// }
/// use wiremock_gen::*;  // this imports generated
/// use wiremock_grpc::*; // this imports MockBuilder
///
/// // ... Later in your test (MyMockServer is generated above)
/// let mut server = MyMockServer::start_default().await;
/// ```
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
            type Response = tonic::codegen::http::Response<tonic::body::Body>;
            type Error = std::convert::Infallible;
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

        impl tonic::server::NamedService for $type {
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
