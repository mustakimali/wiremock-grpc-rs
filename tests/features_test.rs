mod hello {
    tonic::include_proto!("hello");
}

mod wiremock_gen {
    wiremock_grpc::generate!("hello.Greeter", MyMockServer);
}

use hello::{
    greeter_client::GreeterClient, HelloReply, HelloRequest, WeatherReply, WeatherRequest,
};
use std::net::TcpStream;
use tonic::{transport::Channel, Code, Request};
use wiremock_gen::*;
use wiremock_grpc::*;

#[tokio::test]
async fn it_starts_with_specified_port() {
    let server = MyMockServer::start(5055).await;

    assert!(TcpStream::connect(server.address()).is_ok());
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
async fn mocked_header_return() {
    let (mut server, mut client) = create().await;

    // Setup
    let mock = server.setup(
        MockBuilder::given("/hello.Greeter/SayHello")
            .return_header("X-RateLimit-Remaining", "100")
            .return_body(|| HelloReply {
                message: "Hello to you too!".into(),
            }),
    );

    // Act
    let response = client
        .say_hello(HelloRequest {
            name: "Yo yo".into(),
        })
        .await
        .unwrap();

    assert_eq!(
        response
            .metadata()
            .get("X-RateLimit-Remaining")
            .expect("header should be set"),
        "100"
    );
    let _ = server.find_one(&mock);
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
async fn unimplemented_when_mock_not_set() {
    // Server
    let (_, mut client) = create().await;

    // no mock is set up

    // Act
    let response = client
        .say_hello(HelloRequest {
            name: "Yo yo".into(),
        })
        .await;

    assert!(response.is_err());
    assert_eq!(Code::Unimplemented, response.err().unwrap().code());
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

#[tokio::test]
async fn header_discriminated_mocks() {
    let (mut server, mut client) = create().await;

    let mock1_session_id = "mock1";
    let mock2_session_id = "mock2";
    // setup
    let mock1 = server.setup(
        MockBuilder::when()
            .path("/hello.Greeter/SayHello")
            .header("session-id", mock1_session_id)
            .then()
            .return_body(|| HelloReply {
                message: "Hello to you too!".into(),
            }),
    );

    let mock2 = server.setup(
        MockBuilder::when()
            .path("/hello.Greeter/SayHello")
            .header("session-id", mock2_session_id)
            .then()
            .return_body(|| HelloReply {
                message: "Hello to you two!".into(),
            }),
    );

    // Act
    let mut request1 = Request::new(HelloRequest {
        name: "Mustakim".into(),
    });

    request1
        .metadata_mut()
        .insert("session-id", mock1_session_id.parse().unwrap());
    let response1 = client.say_hello(request1).await.unwrap();

    assert_eq!("Hello to you too!", response1.into_inner().message);

    let mut request2 = Request::new(HelloRequest { name: "Zak".into() });
    request2
        .metadata_mut()
        .insert("session-id", mock2_session_id.parse().unwrap());
    let response2 = client.say_hello(request2).await.unwrap();
    assert_eq!("Hello to you two!", response2.into_inner().message);

    // single request
    let tracked_response_1 = server.find_one(&mock1);
    let tracked_response_2 = server.find_one(&mock2);

    assert_eq!(
        tracked_response_1
            .headers
            .get("session-id")
            .expect("header set")
            .to_str()
            .unwrap(),
        mock1_session_id
    );
    assert_eq!(
        tracked_response_2
            .headers
            .get("session-id")
            .expect("header set")
            .to_str()
            .unwrap(),
        mock2_session_id
    );

    assert_eq!(2, server.find_request_count());
}

#[tokio::test]
#[should_panic(expected = "Server terminated with unmatched rules: \n/hello.Greeter/SayHello")]
async fn unmatched_request_panics() {
    let (mut server, _) = create().await;

    // setup
    server.setup(
        MockBuilder::when()
            .path("/hello.Greeter/SayHello")
            .then()
            .return_body(|| HelloReply {
                message: "Hello to you too!".into(),
            }),
    );

    // Act
    // dont send any request

    assert_eq!(0, server.find_request_count());
    assert_eq!(1, server.rules_len());
    assert_eq!(1, server.rules_unmatched());
} // panics

#[allow(dead_code)]
async fn create() -> (MyMockServer, GreeterClient<Channel>) {
    let server = MyMockServer::start_default().await;

    let channel =
        tonic::transport::Channel::from_shared(format!("http://[::1]:{}", server.address().port()))
            .unwrap()
            .connect()
            .await
            .unwrap();
    (server, GreeterClient::new(channel))
}
