mod wiremock_gen {
    wiremock_grpc::generate!("hello.Greeter", MyMockServer);
}

use std::net::TcpStream;

use wiremock_gen::*;
use wiremock_grpc::{
    tonic::{transport::Channel, Code},
    *,
};
use wiremock_grpc_protogen::{
    greeter_client::GreeterClient, HelloReply, HelloRequest, WeatherReply, WeatherRequest,
};

#[tokio::test]
async fn it_starts_with_specified_port() {
    let server = MyMockServer::start(5055).await;

    assert!(TcpStream::connect(&server.address()).is_ok())
}

#[tokio::test]
async fn default() {
    // Server (MyMockServer is generated above)
    let mut server = MyMockServer::start_default().await;

    let request1 = server.setup(
        MockBuilder::when()
            //    👇 RPC prefix
            .path("/hello.Greeter/SayHello")
            .then()
            .return_status(Code::Ok)
            .return_body(|| HelloReply {
                message: "Hello Mustakim".into(),
            }),
    ); // request1 can be used later to inspect the request

    // Client
    // Client code is generated using tonic_build
    let channel =
        tonic::transport::Channel::from_shared(format!("http://[::1]:{}", server.address().port()))
            .unwrap()
            .connect()
            .await
            .unwrap();
    let mut client = GreeterClient::new(channel);

    // Act
    let response = client
        .say_hello(HelloRequest {
            name: "Mustakim".into(),
        })
        .await
        .unwrap();

    assert_eq!("Hello Mustakim", response.into_inner().message);

    // Inspect the request
    // multiple requests
    let requests = server.find(&request1);
    assert!(requests.is_some(), "Request must be logged");
    assert_eq!(1, requests.unwrap().len(), "Only 1 request must be logged");

    // single request
    let request = server.find_one(&request1);
    assert_eq!(
        format!(
            "http://[::1]:{}/hello.Greeter/SayHello",
            server.address().port()
        ),
        request.uri
    );
}

#[tokio::test]
async fn handled_when_mock_set_with_different_status_code() {
    // client & server
    let (mut server, mut client) = create().await;

    server.setup(
        MockBuilder::given("/hello.Greeter/SayHello")
            .return_status(Code::AlreadyExists)
            .return_body(|| HelloReply {
                message: "yo".into(),
            }),
    );

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
    let (_, mut client) = create().await;

    // no mock is set up

    // Act
    let _ = client
        .say_hello(HelloRequest {
            name: "Yo yo".into(),
        })
        .await
        .expect("Must panic");
}

#[tokio::test]
async fn multiple_mocks() {
    let (mut server, mut client) = create().await;

    // setup
    let request1 = server.setup(
        MockBuilder::given("/hello.Greeter/SayHello").return_body(|| HelloReply {
            message: "Hello to you too!".into(),
        }),
    );

    let request2 = server.setup(
        MockBuilder::given("/hello.Greeter/WeatherInfo").return_body(|| WeatherReply {
            weather: "rainy, as always".into(),
        }),
    );

    // Act
    let response1 = client
        .say_hello(HelloRequest {
            name: "Mustakim".into(),
        })
        .await
        .unwrap();

    assert_eq!("Hello to you too!", response1.into_inner().message);

    let response2 = client
        .weather_info(WeatherRequest {
            city: "London".into(),
        })
        .await
        .unwrap();
    assert_eq!("rainy, as always", response2.into_inner().weather);

    // single request
    let _ = server.find_one(&request1);
    let _ = server.find_one(&request2);

    assert_eq!(2, server.find_request_count());
}

async fn create() -> (MyMockServer, GreeterClient<Channel>) {
    let server = MyMockServer::start_default().await;

    let channel =
        tonic::transport::Channel::from_shared(format!("http://[::1]:{}", server.address().port()))
            .unwrap()
            .connect()
            .await
            .unwrap();
    return (server, GreeterClient::new(channel));
}
