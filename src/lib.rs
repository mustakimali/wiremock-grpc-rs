//! # wiremock-grpc-rs
//! Mock gRPC server to test your outgoing gRPC requests.
//! # Example
//! ## Generate Stub
//! ```no_run
//! mod hello_greeter_mock {
//!    wiremock_grpc::generate_stub!("hello.Greeter", Server);
//! }
//! use hello_greeter_mock::*;
//! ```
//! 
//! ## Use it
//! ```no_run
//! #[tokio::test]
//! async fn handled_when_mock_set() {
//!     // Server
//!     let mut server = Server::start_default().await;
//! 
//!     server.setup(
//!         MockBuilder::given("/hello.Greeter/SayHello")
//!             .return_status(Code::Ok)
//!             .return_body(|| HelloReply {
//!                 message: "yo".into(),
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
//!             name: "Yo yo".into(),
//!         })
//!         .await
//!         .unwrap();
//! 
//!     assert_eq!("yo", response.into_inner().message);
//! }
//! ```
mod mock_server;

pub use mock_server::*;