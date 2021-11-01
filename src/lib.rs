//! # wiremock-grpc-rs
//! Mock gRPC server to test your outgoing gRPC requests.
//! # Example
//! ## Generate Stub
//! ```no_run
//! mod hello_greeter_mock {
//!   // hello.Greeter: Is the prefix of all rpc
//!   // MyHelloServer: Arbitrary name of the generated Server
//!    wiremock_grpc::generate_stub!("hello.Greeter", MyHelloServer);
//! }
//! use hello_greeter_mock::*;
//! // MyHelloServer, MockBuilder are available to use
//! // If multiple server are generated then use the
//! // module identifier eg. `hello_greeter_mock::MyHelloServer`
//! ```
//!
//! ## Use it
//! ```no_run
//! #[tokio::test]
//! async fn handled_when_mock_set() {
//!     // Server
//!     let mut server = MyHelloServer::start_default().await;
//! 
//!     server.setup(
//!         MockBuilder::when()
//!             .path("/hello.Greeter/SayHello")
//!             .then()
//!             .return_status(Code::Ok)
//!             .return_body(|| HelloReply {
//!                 message: "Hello Mustakim".into(),
//!             }),
//!     );
//! 
//!     // Client
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
//! }
//! ```
mod mock_server;

pub use mock_server::*;
