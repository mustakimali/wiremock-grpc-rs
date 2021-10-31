use log::debug;
use prost::Message;
use std::{marker::PhantomData, net::{SocketAddr, TcpStream}, sync::{Arc, RwLock}, task::Poll, time::Duration};
use tonic::{Code, codec::Codec, codegen::{http, Body, Never, StdError}};

use crate::greeter_code::{self};

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

        let builder = http::Response::builder()
            .status(200)
            .header("content-type", "application/grpc");

        let path = req.uri().path();
        let inner = self.rules.as_ref();
        let inner = inner.read().unwrap();

        if let Some(req_builder) = inner.iter().find(|x| x.path == path) {
            println!("Matched rule {:?}", req_builder);
            let status = req_builder.status_code.unwrap_or(Code::Ok) as u32;
            println!("Setting status: {}", status);
            let builder = builder.header("grpc-status", format!("{}", status));

            if let Some(body) = &req_builder.result {
                println!("Returning body ({} bytes)", body.len());

                let body = greeter_code::HelloReply {
                    message: "yo".into(),
                };

                let fut = async move {
                    let method = SvcStaticTyped(body);
                    //let method = SvcGeneric(body.encode_to_vec());
                    let codec = tonic::codec::ProstCodec::default();
                    //let codec = GenericCodec::default(); // ToDo: Implement
                    let mut grpc = tonic::server::Grpc::new(codec);
                    let res = grpc.unary(method, req).await;

                    Ok(res)
                };
                return Box::pin(fut);

                /*
                return Box::pin(async move {
                    // let res = async move {greeter_code::HelloReply {
                    //     message: "yo".into(),
                    // }};
                    // let buf = Vec::default();

                    let body = prost::bytes::Bytes::from(body);
                    let body = http_body::Full::new(body);
                    let body =
                        http_body::combinators::BoxBody::new(body).map_err(|err| match err {});
                    let body = tonic::body::BoxBody::new(body);

                    let response = builder.body(body).expect("Set body");

                    Ok(response)
                });
                */
            } else {
                println!("Returning empty body");

                return Box::pin(async move {
                    let body = builder.body(tonic::body::empty_body()).unwrap();
                    Ok(body)
                });
            };
        } else {
            println!("Request unhandled");
            Box::pin(async move {
                Ok(builder
                    .header("grpc-status", "12")
                    .body(tonic::body::empty_body())
                    .unwrap())
            })
        }
    }
}

#[allow(non_camel_case_types)]
struct SvcStaticTyped(greeter_code::HelloReply);
impl tonic::server::UnaryService<greeter_code::HelloRequest> for SvcStaticTyped {
    type Response = greeter_code::HelloReply;
    type Future = tonic::codegen::BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
    fn call(&mut self, _: tonic::Request<greeter_code::HelloRequest>) -> Self::Future {
        let r = self.0.clone();
        let fut = async move { Ok(tonic::Response::new(r)) };

        Box::pin(fut)
    }
}

struct SvcGeneric(Vec<u8>);
impl tonic::server::UnaryService<greeter_code::HelloRequest> for SvcGeneric {
    type Response = http::Response<tonic::body::BoxBody>;
    type Future = tonic::codegen::BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
    fn call(&mut self, _: tonic::Request<greeter_code::HelloRequest>) -> Self::Future {
        let body = self.0.clone();
        let fut = async move { 
            let body = prost::bytes::Bytes::from(body);
            let body = http_body::Full::new(body);
            let body =
                http_body::combinators::BoxBody::new(body).map_err(|err| match err {});
            let body = tonic::body::BoxBody::new(body).boxed_unsync();
            let body = http::Response::new(body);

            Ok(tonic::Response::new(body))
        };

        Box::pin(fut)
    }
}

/*
struct GenericCodec<T, U>{
    _pd: std::marker::PhantomData<(T, U)>,
}
impl<T, U> Default for GenericCodec<T, U> {
    fn default() -> Self {
        Self { _pd: std::marker::PhantomData }
    }
}

impl<T, U> Codec for GenericCodec<T, U>
where
    T: Message + Send + 'static,
    U: Message + Default + Send + 'static,
{
    type Encode = T;
    type Decode = U;

    type Encoder = GenericProstEncoder<T>;
    type Decoder = GenericProstDecoder<U>;

    fn encoder(&mut self) -> Self::Encoder {
        GenericProstEncoder(PhantomData)
    }

    fn decoder(&mut self) -> Self::Decoder {
        todo!()
        //ProstDecoder(PhantomData)
    }
}

/// A [`Encoder`] that knows how to encode `T`.
#[derive(Debug, Clone, Default)]
pub struct GenericProstEncoder<T>(PhantomData<T>);

impl<T: Message> tonic::codec::Encoder for GenericProstDecoder<T> {
    type Item = T;
    type Error = tonic::Status;

    fn encode(&mut self, item: Self::Item, buf: &mut tonic::codec::EncodeBuf<'_>) -> Result<(), Self::Error> {
        item.encode(buf)
            .expect("Message only errors if not enough space");

        Ok(())
    }
}

/// A [`Decoder`] that knows how to decode `U`.
#[derive(Debug, Clone, Default)]
pub struct GenericProstDecoder<U>(PhantomData<U>);

impl<U: Message + Default> tonic::codec::Decoder for GenericProstDecoder<U> {
    type Item = U;
    type Error = tonic::Status;

    fn decode(&mut self, buf: &mut tonic::codec::DecodeBuf<'_>) -> Result<Option<Self::Item>, Self::Error> {
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
*/

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
