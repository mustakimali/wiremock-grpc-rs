mod greeter_code;
pub mod mock_server;

pub use mock_server::*;
#[cfg(test)]
mod tests {
    use std::net::TcpStream;

    use tonic::Code;

    use crate::{
        greeter_code::{self, greeter_client, HelloReply, HelloRequest},
        MockGrpcServer, RequestBuilder,
    };

    #[tokio::test]
    async fn it_starts_with_specified_port() {
        let server = MockGrpcServer::new(50055).start().await;

        assert!(TcpStream::connect(&server.address()).is_ok())
    }

    #[tokio::test]
    async fn it_works() {
        // Server
        let mut server = MockGrpcServer::new(50055).start().await;

        server.setup(
            RequestBuilder::given("/hello.Greeter/SayHello")
                .return_status(Code::Ok)
                .return_body(|| HelloReply {
                    message: "yo".into(),
                }),
        );

        // Client
        let channel = tonic::transport::Channel::from_static("http://[::1]:50055")
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

    struct Msg(dyn prost::Message);

    #[test]
    fn learn_rust() {
        let mut v: Vec<Box<dyn prost::Message>> = Vec::default();

        v.push(Box::new(greeter_code::HelloRequest {
            name: "name".into(),
        }));
        v.push(Box::new(greeter_code::HelloReply {
            message: "message".into(),
        }));

        let r = v.pop().unwrap();
        //let r = r.encode_to_vec();

        //assert_eq!(4, r.len());
    }

    #[test]
    fn create() {
        let cd = std::env::current_dir().unwrap();
        std::env::set_var("OUT_DIR", &cd);
        let cd = cd.join("hello.proto");
        tonic_build::compile_protos(cd).expect("Unable to generate the code");
    }
}
