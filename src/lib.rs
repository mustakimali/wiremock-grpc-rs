mod greeter_code;
pub mod mock_server;

pub use mock_server::*;
#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tonic::Status;

    use crate::{
        greeter_code::{greeter_client, HelloReply, HelloRequest},
        MockGrpcServer, RequestBuilder,
    };

    #[tokio::test]
    async fn it_works() {
        // Server
        let server = MockGrpcServer::new(50055).start();
        tokio::time::sleep(Duration::from_secs(1)).await;

        server.setup(
            RequestBuilder::given("/")
                .return_status(Status::ok(""))
                .return_body(|| {
                    Ok(HelloReply {
                        message: "yo".into(),
                    })
                }),
        );

        // Client
        let channel = tonic::transport::Channel::from_static("http://[::1]:50055")
            .connect()
            .await
            .unwrap();
        let mut client = greeter_client::GreeterClient::new(channel);

        // Act
        let _r = client
            .say_hello(HelloRequest {
                name: "Yo yo".into(),
            })
            .await
            .unwrap();
    }

    #[test]
    fn create() {
        let cd = std::env::current_dir().unwrap();
        std::env::set_var("OUT_DIR", &cd);
        let cd = cd.join("hello.proto");
        tonic_build::compile_protos(cd).expect("Unable to generate the code");
    }
}
