use tonic::Code;

mod wiremock_gen {
    wiremock_grpc::generate!("hello.Greeter", MyMockServer);
}

use wiremock_gen::*;
use wiremock_grpc::*;
use wiremock_grpc_protogen::HelloReply;

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
