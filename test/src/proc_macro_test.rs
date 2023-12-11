use wiremock_grpc::tonic::transport::Channel;
use wiremock_grpc_proc_macro::WireMockGrpcServer;
use wiremock_grpc_protogen::{greeter_client, greeter_server::{self, Greeter}};

#[derive(WireMockGrpcServer)]
struct Server<T> where T : Greeter {
    #[server] field: greeter_server::GreeterServer<T>,
    name: String,
}

pub fn get_type_name_from_token_stream() -> String {
    todo!()
}

#[test]
fn reflection_works() {
    
}