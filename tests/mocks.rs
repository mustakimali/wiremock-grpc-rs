use tonic::Code;

mod my_mock {
    wiremock_grpc::generate!("hello.Greeter", MyMockServer);
}
use wiremock_grpc::*;
use my_mock::*;

/// The response message containing the greetings
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HelloReply {
    #[prost(string, tag = "1")]
    pub message: ::prost::alloc::string::String,
}

#[tokio::test]
async fn mock_builder() {
    let mut server = MyMockServer::start_default().await;

    server.setup(
        MockBuilder::when()
            .path("/")
            .then()
            .return_status(Code::AlreadyExists),
    );

    server.setup(
        MockBuilder::when()
            .path("/")
            .then()
            .return_status(Code::AlreadyExists)
            .return_body(|| HelloReply {
                message: "Hello".into(),
            }),
    );
}
