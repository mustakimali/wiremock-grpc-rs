use prost::{bytes::BufMut, Message};

use tonic::{codec::Codec, Code};

pub(crate) struct GenericSvc(pub(crate) Vec<u8>);
impl tonic::server::UnaryService<Vec<u8>> for GenericSvc {
    type Response = Vec<u8>;
    type Future = tonic::codegen::BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
    fn call(&mut self, _: tonic::Request<Vec<u8>>) -> Self::Future {
        let body = self.0.clone();
        let fut = async move { Ok(tonic::Response::new(body)) };

        Box::pin(fut)
    }
}

#[derive(Default)]
pub(crate) struct GenericCodec;

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

        // copy the respone body `item` to the buffer
        for i in item {
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
