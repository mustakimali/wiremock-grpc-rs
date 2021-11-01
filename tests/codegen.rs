use tonic::Code;

mod wiremock_gen {
    wiremock_grpc::generate!("hello.Greeter", Server);
}
use wiremock_gen::*;
use wiremock_grpc::*;

#[tokio::test]
async fn codegen_works() {
    let mut server = Server::start_default().await;

    server.setup(
        MockBuilder::when()
            .path("")
            .then()
            .return_status(Code::Aborted),
    );

    assert!(std::net::TcpStream::connect(&server.address()).is_ok())
}
