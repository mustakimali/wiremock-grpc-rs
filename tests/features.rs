use std::net::TcpStream;

use tonic::Code;
use wiremock_grpc_rs::*;

use example::{greeter_client::GreeterClient, *};

#[tokio::test]
async fn it_starts_with_specified_port() {
    let server = MockGrpcServer::new(5055).start().await;

    assert!(TcpStream::connect(&server.address()).is_ok())
}

#[tokio::test]
async fn handled_when_mock_set() {
    // Server
    let mut server = MockGrpcServer::start_default().await;

    server.setup(
        MockBuilder::given("/hello.Greeter/SayHello")
            .return_status(Code::Ok)
            .return_body(|| HelloReply {
                message: "yo".into(),
            }),
    );

    // Client
    let channel =
        tonic::transport::Channel::from_shared(format!("http://[::1]:{}", server.address().port()))
            .unwrap()
            .connect()
            .await
            .unwrap();
    let mut client = GreeterClient::new(channel);

    // Act
    let response = client
        .say_hello(HelloRequest {
            name: "Yo yo".into(),
        })
        .await
        .unwrap();

    assert_eq!("yo", response.into_inner().message);
}

#[tokio::test]
async fn handled_when_mock_set_with_different_status_code() {
    // Server
    let mut server = MockGrpcServer::start_default().await;

    server.setup(
        MockBuilder::given("/hello.Greeter/SayHello")
            .return_status(Code::AlreadyExists)
            .return_body(|| HelloReply {
                message: "yo".into(),
            }),
    );

    // Client
    let channel =
        tonic::transport::Channel::from_shared(format!("http://[::1]:{}", server.address().port()))
            .unwrap()
            .connect()
            .await
            .unwrap();
    let mut client = GreeterClient::new(channel);

    // Act
    let response = client
        .say_hello(HelloRequest {
            name: "Yo yo".into(),
        })
        .await;

    assert!(response.is_err());
    assert_eq!(Code::AlreadyExists, response.err().unwrap().code());
}

#[tokio::test]
#[should_panic]
async fn panic_when_mock_not_set() {
    // Server
    let server = MockGrpcServer::start_default().await;

    // no mock is set up

    // Client
    let channel =
        tonic::transport::Channel::from_shared(format!("http://[::1]:{}", server.address().port()))
            .unwrap()
            .connect()
            .await
            .unwrap();
    let mut client = GreeterClient::new(channel);

    // Act
    let _ = client
        .say_hello(HelloRequest {
            name: "Yo yo".into(),
        })
        .await
        .expect("Must panic");
}

//#[test]
#[allow(dead_code)]
fn create() {
    let cd = std::env::current_dir().unwrap();
    std::env::set_var("OUT_DIR", &cd);
    let cd = cd.join("hello.proto");
    tonic_build::compile_protos(cd).expect("Unable to generate the code");
}

mod example {
    /// The request message containing the user's name.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct HelloRequest {
        #[prost(string, tag = "1")]
        pub name: ::prost::alloc::string::String,
    }
    /// The response message containing the greetings
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct HelloReply {
        #[prost(string, tag = "1")]
        pub message: ::prost::alloc::string::String,
    }
    #[doc = r" Generated client implementations."]
    pub mod greeter_client {
        #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
        use tonic::codegen::*;
        #[doc = " The greeting service definition."]
        #[derive(Debug, Clone)]
        pub struct GreeterClient<T> {
            inner: tonic::client::Grpc<T>,
        }
        impl GreeterClient<tonic::transport::Channel> {
            #[doc = r" Attempt to create a new client by connecting to a given endpoint."]
            pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
            where
                D: std::convert::TryInto<tonic::transport::Endpoint>,
                D::Error: Into<StdError>,
            {
                let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
                Ok(Self::new(conn))
            }
        }
        impl<T> GreeterClient<T>
        where
            T: tonic::client::GrpcService<tonic::body::BoxBody>,
            T::ResponseBody: Body + Send + 'static,
            T::Error: Into<StdError>,
            <T::ResponseBody as Body>::Error: Into<StdError> + Send,
        {
            pub fn new(inner: T) -> Self {
                let inner = tonic::client::Grpc::new(inner);
                Self { inner }
            }
            pub fn with_interceptor<F>(
                inner: T,
                interceptor: F,
            ) -> GreeterClient<InterceptedService<T, F>>
            where
                F: tonic::service::Interceptor,
                T: tonic::codegen::Service<
                    http::Request<tonic::body::BoxBody>,
                    Response = http::Response<
                        <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                    >,
                >,
                <T as tonic::codegen::Service<http::Request<tonic::body::BoxBody>>>::Error:
                    Into<StdError> + Send + Sync,
            {
                GreeterClient::new(InterceptedService::new(inner, interceptor))
            }
            #[doc = r" Compress requests with `gzip`."]
            #[doc = r""]
            #[doc = r" This requires the server to support it otherwise it might respond with an"]
            #[doc = r" error."]
            pub fn send_gzip(mut self) -> Self {
                self.inner = self.inner.send_gzip();
                self
            }
            #[doc = r" Enable decompressing responses with `gzip`."]
            pub fn accept_gzip(mut self) -> Self {
                self.inner = self.inner.accept_gzip();
                self
            }
            #[doc = " Sends a greeting"]
            pub async fn say_hello(
                &mut self,
                request: impl tonic::IntoRequest<super::HelloRequest>,
            ) -> Result<tonic::Response<super::HelloReply>, tonic::Status> {
                self.inner.ready().await.map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
                let codec = tonic::codec::ProstCodec::default();
                let path = http::uri::PathAndQuery::from_static("/hello.Greeter/SayHello");
                self.inner.unary(request.into_request(), path, codec).await
            }
        }
    }

    #[doc = r" Generated server implementations."]
    pub mod greeter_server {
        #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
        use tonic::codegen::*;
        #[doc = "Generated trait containing gRPC methods that should be implemented for use with GreeterServer."]
        #[async_trait]
        pub trait Greeter: Send + Sync + 'static {
            #[doc = " Sends a greeting"]
            async fn say_hello(
                &self,
                request: tonic::Request<super::HelloRequest>,
            ) -> Result<tonic::Response<super::HelloReply>, tonic::Status>;
        }
        #[doc = " The greeting service definition."]
        #[derive(Debug)]
        pub struct GreeterServer<T: Greeter> {
            inner: _Inner<T>,
            accept_compression_encodings: (),
            send_compression_encodings: (),
        }
        struct _Inner<T>(Arc<T>);
        impl<T: Greeter> GreeterServer<T> {
            pub fn new(inner: T) -> Self {
                let inner = Arc::new(inner);
                let inner = _Inner(inner);
                Self {
                    inner,
                    accept_compression_encodings: Default::default(),
                    send_compression_encodings: Default::default(),
                }
            }
            pub fn with_interceptor<F>(inner: T, interceptor: F) -> InterceptedService<Self, F>
            where
                F: tonic::service::Interceptor,
            {
                InterceptedService::new(Self::new(inner), interceptor)
            }
        }
        impl<T, B> tonic::codegen::Service<http::Request<B>> for GreeterServer<T>
        where
            T: Greeter,
            B: Body + Send + 'static,
            B::Error: Into<StdError> + Send + 'static,
        {
            type Response = http::Response<tonic::body::BoxBody>;
            type Error = Never;
            type Future = BoxFuture<Self::Response, Self::Error>;
            fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
                Poll::Ready(Ok(()))
            }
            fn call(&mut self, req: http::Request<B>) -> Self::Future {
                let inner = self.inner.clone();
                match req.uri().path() {
                    "/hello.Greeter/SayHello" => {
                        #[allow(non_camel_case_types)]
                        struct SayHelloSvc<T: Greeter>(pub Arc<T>);
                        impl<T: Greeter> tonic::server::UnaryService<super::HelloRequest> for SayHelloSvc<T> {
                            type Response = super::HelloReply;
                            type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                            fn call(
                                &mut self,
                                request: tonic::Request<super::HelloRequest>,
                            ) -> Self::Future {
                                let inner = self.0.clone();
                                let fut = async move {
                                    let r = (*inner).say_hello(request).await;
                                    r
                                };
                                Box::pin(fut)
                            }
                        }
                        let accept_compression_encodings = self.accept_compression_encodings;
                        let send_compression_encodings = self.send_compression_encodings;
                        let inner = self.inner.clone();
                        let fut = async move {
                            let inner = inner.0;
                            let method = SayHelloSvc(inner);
                            let codec = tonic::codec::ProstCodec::default();
                            let mut grpc = tonic::server::Grpc::new(codec)
                                .apply_compression_config(
                                    accept_compression_encodings,
                                    send_compression_encodings,
                                );
                            let res = grpc.unary(method, req).await;
                            Ok(res)
                        };
                        Box::pin(fut)
                    }
                    _ => Box::pin(async move {
                        Ok(http::Response::builder()
                            .status(200)
                            .header("grpc-status", "12")
                            .header("content-type", "application/grpc")
                            .body(empty_body())
                            .unwrap())
                    }),
                }
            }
        }
        impl<T: Greeter> Clone for GreeterServer<T> {
            fn clone(&self) -> Self {
                let inner = self.inner.clone();
                Self {
                    inner,
                    accept_compression_encodings: self.accept_compression_encodings,
                    send_compression_encodings: self.send_compression_encodings,
                }
            }
        }
        impl<T: Greeter> Clone for _Inner<T> {
            fn clone(&self) -> Self {
                Self(self.0.clone())
            }
        }
        impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:?}", self.0)
            }
        }
        impl<T: Greeter> tonic::transport::NamedService for GreeterServer<T> {
            const NAME: &'static str = "hello.Greeter";
        }
    }
}
