//! # wiremock-grpc-rs
//! Mock gRPC server to test your outgoing gRPC requests.
//! ## Example
//! ```
//! use wiremock_grpc_rs::{MockGrpcServer, RequestBuilder};
//! use crate::{example::{greeter_client, HelloRequest, HelloReply}};
//! use tonic::Code;
//!
//! tokio_test::block_on(async {
//!  // Server
//! let mut server = MockGrpcServer::start_default().await;
//!
//! server.setup(
//!     RequestBuilder::given("/hello.Greeter/SayHello")
//!         .return_status(Code::Ok)
//!         .return_body(|| HelloReply {
//!             message: "yo".into(),
//!         }),
//! );
//!
//! // Client
//! let channel = tonic::transport::Channel::from_shared(format!(
//!     "http://[::1]:{}",
//!     server.address().port()
//! ))
//! .unwrap()
//! .connect()
//! .await
//! .unwrap();
//! let mut client = example::GreeterClient::new(channel);
//!
//! // Act
//! let response = client
//!     .say_hello(HelloRequest {
//!         name: "Yo yo".into(),
//!     })
//!     .await
//!     .unwrap();
//!
//! assert_eq!("yo", response.into_inner().message);
//! });
//!  ```
mod example;
mod mock_server;

pub use mock_server::{MockGrpcServer, RequestBuilder};

#[cfg(test)]
mod tests {
    use std::net::TcpStream;

    use tonic::Code;

    use crate::{
        example::{greeter_client, HelloReply, HelloRequest},
        MockGrpcServer, RequestBuilder,
    };

    #[tokio::test]
    async fn it_starts_with_specified_port() {
        let server = MockGrpcServer::new(5055).start().await;

        assert!(TcpStream::connect(&server.address()).is_ok())
    }

    #[tokio::test]
    async fn handled_when_mock_set() {
        // Server
        let mut server = MockGrpcServer::start_default().await;

        server.setup(
            RequestBuilder::given("/hello.Greeter/SayHello")
                .return_status(Code::Ok)
                .return_body(|| HelloReply {
                    message: "yo".into(),
                }),
        );

        // Client
        let channel = tonic::transport::Channel::from_shared(format!(
            "http://[::1]:{}",
            server.address().port()
        ))
        .unwrap()
        .connect()
        .await
        .unwrap();
        let mut client = greeter_client::GreeterClient::new(channel);

        // Act
        let response = client
            .say_hello(HelloRequest {
                name: "Yo yo".into(),
            })
            .await
            .unwrap();

        assert_eq!("yo", response.into_inner().message);
    }

    #[tokio::test]
    async fn handled_when_mock_set_with_different_status_code() {
        // Server
        let mut server = MockGrpcServer::start_default().await;

        server.setup(
            RequestBuilder::given("/hello.Greeter/SayHello")
                .return_status(Code::AlreadyExists)
                .return_body(|| HelloReply {
                    message: "yo".into(),
                }),
        );

        // Client
        let channel = tonic::transport::Channel::from_shared(format!(
            "http://[::1]:{}",
            server.address().port()
        ))
        .unwrap()
        .connect()
        .await
        .unwrap();
        let mut client = greeter_client::GreeterClient::new(channel);

        // Act
        let response = client
            .say_hello(HelloRequest {
                name: "Yo yo".into(),
            })
            .await;

        assert!(response.is_err());
        assert_eq!(Code::AlreadyExists, response.err().unwrap().code());
    }

    #[tokio::test]
    #[should_panic]
    async fn panic_when_mock_not_set() {
        // Server
        let server = MockGrpcServer::start_default().await;

        // no mock is set up

        // Client
        let channel = tonic::transport::Channel::from_shared(format!(
            "http://[::1]:{}",
            server.address().port()
        ))
        .unwrap()
        .connect()
        .await
        .unwrap();
        let mut client = greeter_client::GreeterClient::new(channel);

        // Act
        let _ = client
            .say_hello(HelloRequest {
                name: "Yo yo".into(),
            })
            .await
            .expect("Must panic");
    }

    //#[test]
    #[allow(dead_code)]
    fn create() {
        let cd = std::env::current_dir().unwrap();
        std::env::set_var("OUT_DIR", &cd);
        let cd = cd.join("hello.proto");
        tonic_build::compile_protos(cd).expect("Unable to generate the code");
    }
}
