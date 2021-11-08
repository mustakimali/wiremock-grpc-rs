use wiremock_grpc_proc_macro::WireMockGrpcServer;
use wiremock_grpc_protogen::greeter_server::{self, Greeter};


#[derive(WireMockGrpcServer)]
struct Server {
    #[server] field: String //greeter_server::GreeterServer
}

#[test]
fn works() {

}