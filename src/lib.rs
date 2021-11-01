//! # wiremock-grpc-rs
//! Mock gRPC server to test your outgoing gRPC requests.
//! # Example
//! ## Quick start
//! ```no_run
//! use wiremock_grpc::*;
//! use tonic::transport::Channel;
//!
//! let mut server = tokio_test::block_on(MockGrpcServer::start_default());
//! server.setup(
//!         MockBuilder::when()
//!            // one or more conditions
//!            .path("/")
//!            .then()
//!            // one or more mock
//!            .return_body(||HelloReply {message: "Welcome Mustakim!".into()})
//! );
//!
//! // Client
//! let channel = Channel::from_shared(format!("http://[::1]:{}", server.address().port()))
//!     .unwrap()
//!     .connect()
//!     .await
//!     .unwrap();
//! let mut client = GreeterClient::new(channel);
//!
//! // Act
//! let response = client
//! .say_hello(HelloRequest {
//!     name: "Mustakim".into(),
//! })
//! .await
//! .unwrap();
//!
//! // Assert
//! assert_eq!("Welcome Mustakim!", response.into_inner().message);
//! ```
//!
//! ## Starting the server
//! There are two ways to start a mock gRPC server.!
//! ```
//! use wiremock_grpc_rs::MockGrpcServer;
//! use std::net::TcpStream;
//!
//! // Using a given port
//! let mut server = tokio_test::block_on(MockGrpcServer::new(5055).start());
//! assert!(std::net::TcpStream::connect(&server.address()).is_ok());
//!
//! // Using an unused port
//! let mut server = tokio_test::block_on(MockGrpcServer::start_default());
//! assert!(std::net::TcpStream::connect(&server.address()).is_ok());
//! ```
mod builder;
mod mock_server;
mod tonic_ext;

pub use builder::{MockBuilder, Mountable, Then};
pub use mock_server::MockGrpcServer;
