use tonic::Code;
use wiremock_grpc_rs::*;

/// The response message containing the greetings
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HelloReply {
    #[prost(string, tag = "1")]
    pub message: ::prost::alloc::string::String,
}

#[tokio::test]
async fn mock_builder() {
    let mut server = MockGrpcServer::start_default().await;

    server.setup(
        RequestBuilder::when()
            .path("/")
            .then()
            .return_status(Code::AlreadyExists),
    );

    server.setup(
        RequestBuilder::when()
            .path("/")
            .then()
            .return_status(Code::AlreadyExists)
            .return_body(|| HelloReply {
                message: "Hello".into(),
            }),
    );
}
