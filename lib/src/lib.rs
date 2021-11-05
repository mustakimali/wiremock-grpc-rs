//! Mock gRPC server to test your outgoing gRPC requests.
//! # Example
//! ## Quick start
//! ```no_run
//! mod wiremock_gen {
//!     // hello.Greeter: is the prefix of all rpc,
//!     // MyMockServer: name of the generated Server,
//!     wiremock_grpc::generate!("hello.Greeter", MyMockServer);
//! }
//! use wiremock_gen::*;  // this imports generated
//! use wiremock_grpc::*; // this imports MockBuilder
//!
//! #[tokio::test]
//! async fn default() {
//!     // Server (MyMockServer is generated above)
//!     let mut server = MyMockServer::start_default().await;
//!
//!     let request1 = server.setup(
//!         MockBuilder::when()
//!             //    👇 RPC prefix
//!             .path("/hello.Greeter/SayHello")
//!             .then()
//!             .return_status(Code::Ok)
//!             .return_body(|| HelloReply {
//!                 message: "Hello Mustakim".into(),
//!             }),
//!     ); // request1 can be used later to inspect the request
//!
//!     // Client
//!     // Client code is generated using tonic_build
//!     let channel =
//!         tonic::transport::Channel::from_shared(format!("http://[::1]:{}", server.address().port()))
//!             .unwrap()
//!             .connect()
//!             .await
//!             .unwrap();
//!     let mut client = GreeterClient::new(channel);
//!
//!     // Act
//!     let response = client
//!         .say_hello(HelloRequest {
//!             name: "Mustakim".into(),
//!         })
//!         .await
//!         .unwrap();
//!
//!     assert_eq!("Hello Mustakim", response.into_inner().message);
//!
//!     // Inspect the request
//!     // multiple requests
//!     let requests = server.find(&request1);
//!     assert!(requests.is_some(), "Request must be logged");
//!     assert_eq!(1, requests.unwrap().len(), "Only 1 request must be logged");
//!
//!     // single request
//!     let request = server.find_one(&request1);
//!     assert_eq!(
//!         format!(
//!             "http://[::1]:{}/hello.Greeter/SayHello",
//!             server.address().port()
//!         ),
//!         request.uri
//!     );
//! }
//! ```

mod builder;
mod codegen;
mod invocations;
mod grpc_server;
mod tonic_ext;

pub use builder::{MockBuilder, Mountable, Then};
pub use grpc_server::GrpcServer;

pub extern crate http_body;
pub extern crate tonic;
