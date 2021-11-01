use prost::{bytes::BufMut, Message};
use std::{ops::Deref, task::Poll};
use tonic::{
    codec::Codec,
    codegen::{
        http::{self, HeaderValue},
        Body, Never, StdError,
    },
    Code,
};

use crate::MockGrpcServer;

//---------------
// GEN CODES    |
//---------------
#[derive(Clone)]
pub struct Server(pub(crate) MockGrpcServer);

impl Deref for Server {
    type Target = MockGrpcServer;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<B> tonic::codegen::Service<tonic::codegen::http::Request<B>> for Server
where
    B: Body + Send + 'static,
    B::Error: Into<StdError> + Send + 'static,
{
    type Response = tonic::codegen::http::Response<tonic::body::BoxBody>;
    type Error = tonic::codegen::Never;
    type Future = tonic::codegen::BoxFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: http::Request<B>) -> Self::Future {
        self.0.handle_request(req)
    }
}

impl tonic::transport::NamedService for Server {
    const NAME: &'static str = "hello.Greeter";
}

impl Server {
    pub async fn start_default() -> Self {
        let port = MockGrpcServer::find_unused_port()
            .await
            .expect("Unable to find an open port");

        Self(MockGrpcServer::new(port)).start(port).await
    }

    pub async fn start(&self, port: u16) -> Self {
        let grpc_serve = MockGrpcServer::new(port);
        let address = grpc_serve.address().clone();
        let grpc_server = grpc_serve
            .start(|| {
                tokio::spawn(
                    tonic::transport::Server::builder()
                        .add_service(self.clone())
                        .serve(address),
                )
            })
            .await;
        Self(grpc_server)
    }
}

//---------------
// GEN CODES    |
//---------------

impl MockGrpcServer {
    fn handle_request<B>(
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

struct SvcGeneric(Vec<u8>);
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
