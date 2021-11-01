#[macro_export]
macro_rules! generate_stub {
    ($name:literal, $t:ident) => {
        use std::{
            net::{SocketAddr},
            sync::{Arc, RwLock},
            time::Duration,
        };

        use rand::Rng;
        use std::net::TcpStream;
        use prost::{bytes::BufMut, Message};
        use std::task::Poll;
        use tonic::{
            codec::Codec,
            Code,
            codegen::{
                http::{self, HeaderValue},
                Body, Never, StdError,
            },
        };

        /// A running gRPC server that binds to service with prefix: `
        #[doc = $name]
        /// `
        /// # Example
        /// ```no_run
        /// let mut server = 
        #[doc = std::stringify!($t)]
        /// ::start_default().await;
        /// ```
        #[derive(Clone)]
        pub struct $t {
            address: SocketAddr,
            inner: Arc<Option<Inner>>,
            pub(crate) rules: Arc<RwLock<Vec<MockBuilder>>>,
        }

        struct Inner {
            #[allow(dead_code)]
            join_handle: tokio::task::JoinHandle<Result<(), tonic::transport::Error>>,
        }

        impl Drop for $t {
            fn drop(&mut self) {
                if let Some(r) = self.inner.as_ref() {
                    println!("Terminating server");
                    drop(&r.join_handle);
                }
            }
        }

        impl $t {
            pub fn new(port: u16) -> Self {
                Self {
                    address: format!("[::1]:{}", port).parse().unwrap(),
                    inner: Arc::default(),
                    rules: Arc::default(),
                }
            }

            pub async fn start_default() -> Self {
                let port = $t::find_unused_port()
                    .await
                    .expect("Unable to find an open port");

                $t::new(port).start().await
            }

            async fn find_unused_port() -> Option<u16> {
                let mut rng = rand::thread_rng();

                loop {
                    let port: u16 = rng.gen_range(50000..60000);
                    let addr: SocketAddr = format!("[::1]:{}", port).parse().unwrap();

                    if !TcpStream::connect_timeout(&addr, std::time::Duration::from_millis(25))
                        .is_ok()
                    {
                        return Some(port);
                    }
                    tokio::time::sleep(Duration::from_millis(25)).await;
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
                    if TcpStream::connect_timeout(
                        &self.address,
                        std::time::Duration::from_millis(25),
                    )
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

            pub fn setup<M>(&mut self, r: M) -> $t
            where
                M: Then + Mountable,
            {
                r.mount(self);

                self.to_owned()
            }

            pub fn address(&self) -> &SocketAddr {
                &self.address
            }
        }

        impl tonic::transport::NamedService for $t {
            const NAME: &'static str = $name;
        }

        impl<B> tonic::codegen::Service<http::Request<B>> for $t
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
                    let code = req_builder.status_code.unwrap_or(Code::Ok);

                    if let Some(body) = &req_builder.result {
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

        pub(crate) struct SvcGeneric(Vec<u8>);
        impl tonic::server::UnaryService<Vec<u8>> for SvcGeneric {
            type Response = Vec<u8>;
            type Future = tonic::codegen::BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
            fn call(&mut self, _: tonic::Request<Vec<u8>>) -> Self::Future {
                let body = self.0.clone();
                let fut = async move { Ok(tonic::Response::new(body)) };

                Box::pin(fut)
            }
        }

        struct GenericCodec;

        impl Default for GenericCodec {
            fn default() -> Self {
                Self {}
            }
        }

        impl Codec for GenericCodec {
            type Encode = Vec<u8>;
            type Decode = Vec<u8>;

            type Encoder = GenericProstEncoder;
            type Decoder = GenericProstDecoder;

            fn encoder(&mut self) -> Self::Encoder {
                GenericProstEncoder(Vec::default())
            }

            fn decoder(&mut self) -> Self::Decoder {
                GenericProstDecoder {}
            }
        }

        /// A [`Encoder`] that knows how to encode `T`.
        #[derive(Debug, Clone, Default)]
        pub struct GenericProstEncoder(Vec<u8>);

        impl tonic::codec::Encoder for GenericProstEncoder {
            type Item = Vec<u8>;
            type Error = tonic::Status;

            fn encode(
                &mut self,
                item: Self::Item,
                buf: &mut tonic::codec::EncodeBuf<'_>,
            ) -> Result<(), Self::Error> {
                // construct the BytesMut from the Vec<u8>
                let mut b = prost::bytes::BytesMut::new();
                for i in item {
                    b.put_u8(i);
                }

                // copy to buffer
                for i in b {
                    buf.put_u8(i);
                }

                Ok(())
            }
        }

        /// A [`Decoder`] that knows how to decode `U`.
        #[derive(Debug, Clone, Default)]
        pub struct GenericProstDecoder;

        impl tonic::codec::Decoder for GenericProstDecoder {
            type Item = Vec<u8>;
            type Error = tonic::Status;

            fn decode(
                &mut self,
                buf: &mut tonic::codec::DecodeBuf<'_>,
            ) -> Result<Option<Self::Item>, Self::Error> {
                let item = Message::decode(buf)
                    .map(Option::Some)
                    .map_err(from_decode_error)?;

                Ok(item)
            }
        }

        fn from_decode_error(error: prost::DecodeError) -> tonic::Status {
            // Map Protobuf parse errors to an INTERNAL status code, as per
            // https://github.com/grpc/grpc/blob/master/doc/statuscodes.md
            tonic::Status::new(Code::Internal, error.to_string())
        }

            pub trait Then {
                fn return_status(self, status: tonic::Code) -> Self;

                fn return_body<T, F>(self, f: F) -> Self
                where
                    F: Fn() -> T,
                    T: prost::Message;
            }

            pub trait Mountable {
                fn mount(self, s: &mut $t);
            }

            /// Builder pattern to set up a mock response for a given request.
            #[derive(Debug)]
            pub struct MockBuilder {
                pub(crate) path: String,
                pub(crate) status_code: Option<tonic::Code>,
                pub(crate) result: Option<Vec<u8>>,
            }

            pub struct WhenBuilder {
                path: Option<String>,
            }
            impl WhenBuilder {
                pub fn path(&self, p: &str) -> Self {
                    Self {
                        path: Some(p.into()),
                    }
                }

                pub fn then(&self) -> ThenBuilder {
                    self.validate();
                    ThenBuilder {
                        path: self.path.clone().unwrap(),
                        status_code: None,
                        result: None,
                    }
                }

                fn validate(&self) {
                    self.path.as_ref().expect(
                        "You must set one or more condition to match (eg. `.when().path(/* ToDo */).then()`)",
                    );
                }
            }

            pub struct ThenBuilder {
                path: String,
                status_code: Option<tonic::Code>,
                result: Option<Vec<u8>>,
            }

            impl MockBuilder {
                pub fn given(path: &str) -> Self {
                    Self {
                        path: path.into(),
                        result: None,
                        status_code: None,
                    }
                }

                pub fn when() -> WhenBuilder {
                    WhenBuilder { path: None }
                }
            }

            impl Mountable for MockBuilder {
                fn mount(self, s: &mut $t) {
                    if self.status_code.is_none() && self.result.is_none() {
                        panic!("Must set the status code or body before attempting to mount the rule.");
                    }

                    s.rules.write().unwrap().push(self);
                }
            }

            impl Then for MockBuilder {
                fn return_status(self, status: tonic::Code) -> Self {
                    Self {
                        status_code: Some(status),
                        ..self
                    }
                }

                fn return_body<T, F>(self, f: F) -> Self
                where
                    F: Fn() -> T,
                    T: prost::Message,
                {
                    let result = f();
                    let mut buf = prost::bytes::BytesMut::new();
                    let _ = result
                        .encode(&mut buf)
                        .expect("Unable to encode the message");
                    let result = buf.to_vec();

                    Self {
                        result: Some(result),
                        ..self
                    }
                }
            }

            impl Then for ThenBuilder {
                fn return_status(self, status: tonic::Code) -> Self {
                    Self {
                        status_code: Some(status),
                        ..self
                    }
                }

                fn return_body<T, F>(self, f: F) -> Self
                where
                    F: Fn() -> T,
                    T: prost::Message,
                {
                    let result = f();
                    let mut buf = prost::bytes::BytesMut::new();
                    let _ = result
                        .encode(&mut buf)
                        .expect("Unable to encode the message");
                    let result = buf.to_vec();

                    Self {
                        result: Some(result),
                        ..self
                    }
                }
            }

            impl Into<MockBuilder> for ThenBuilder {
                fn into(self) -> MockBuilder {
                    MockBuilder {
                        path: self.path,
                        status_code: self.status_code,
                        result: self.result,
                    }
                }
            }

            impl Mountable for ThenBuilder {
                fn mount(self, s: &mut $t) {
                    let rb: MockBuilder = self.into();

                    rb.mount(s)
                }
            }
    };
}
