mod hello {
    tonic::include_proto!("hello");
}

wiremock_grpc::generate_svc! {
    package hello;
    service Greeter {
        SayHello,
        WeatherInfo,
    }
}

use hello::{
    greeter_client::GreeterClient, HelloReply, HelloRequest, WeatherReply, WeatherRequest,
};
use tonic::{transport::Channel, Code};
use wiremock_grpc::*;

#[tokio::test]
async fn proc_macro_rpc_method() {
    let mut server = GreeterMockServer::start_default().await;

    server.setup(
        MockBuilder::when()
            .path_say_hello()
            .then()
            .return_status(Code::Ok)
            .return_body(|| HelloReply {
                message: "Hello from proc macro!".into(),
            }),
    );

    let channel =
        tonic::transport::Channel::from_shared(format!("http://[::1]:{}", server.address().port()))
            .unwrap()
            .connect()
            .await
            .unwrap();
    let mut client = GreeterClient::new(channel);

    let response = client
        .say_hello(HelloRequest {
            name: "Test".into(),
        })
        .await
        .unwrap();

    assert_eq!("Hello from proc macro!", response.into_inner().message);
}

#[tokio::test]
async fn proc_macro_multiple_rpc_methods() {
    let (mut server, mut client) = create().await;

    server.setup(
        MockBuilder::when()
            .path_say_hello()
            .then()
            .return_body(|| HelloReply {
                message: "Hello!".into(),
            }),
    );

    server.setup(
        MockBuilder::when()
            .path_weather_info()
            .then()
            .return_body(|| WeatherReply {
                weather: "Sunny? you wish it was, but it's actually raining".into(),
            }),
    );

    let response1 = client
        .say_hello(HelloRequest {
            name: "Test".into(),
        })
        .await
        .unwrap();
    assert_eq!("Hello!", response1.into_inner().message);

    let response2 = client
        .weather_info(WeatherRequest {
            city: "London".into(),
        })
        .await
        .unwrap();
    assert_eq!(
        "Sunny? you wish it was, but it's actually raining",
        response2.into_inner().weather
    );
}

#[tokio::test]
async fn proc_macro_with_headers() {
    let (mut server, mut client) = create().await;

    server.setup(
        MockBuilder::when()
            .path_say_hello()
            .header("x-custom-header", "test-value")
            .then()
            .return_body(|| HelloReply {
                message: "With header!".into(),
            }),
    );

    let mut request = tonic::Request::new(HelloRequest {
        name: "Test".into(),
    });
    request
        .metadata_mut()
        .insert("x-custom-header", "test-value".parse().unwrap());

    let response = client.say_hello(request).await.unwrap();
    assert_eq!("With header!", response.into_inner().message);
}

mod custom_server_test {
    use super::hello::{greeter_client::GreeterClient, HelloReply, HelloRequest};
    use wiremock_grpc::*;

    wiremock_grpc::generate_svc! {
        package hello;
        service Greeter as CustomServer {
            SayHello,
        }
    }

    #[tokio::test]
    async fn proc_macro_custom_server_name() {
        let mut server = CustomServer::start_default().await;

        server.setup(
            MockBuilder::when()
                .path_say_hello()
                .then()
                .return_body(|| HelloReply {
                    message: "Custom server!".into(),
                }),
        );

        let channel = tonic::transport::Channel::from_shared(format!(
            "http://[::1]:{}",
            server.address().port()
        ))
        .unwrap()
        .connect()
        .await
        .unwrap();
        let mut client = GreeterClient::new(channel);

        let response = client
            .say_hello(HelloRequest {
                name: "Test".into(),
            })
            .await
            .unwrap();

        assert_eq!("Custom server!", response.into_inner().message);
    }
}

mod hello_extended {
    tonic::include_proto!("hello.extended");
}

mod nested_package_test {
    use super::hello_extended::{
        extended_greeter_client::ExtendedGreeterClient, HelloReply, HelloRequest,
    };
    use wiremock_grpc::*;

    wiremock_grpc::generate_svc! {
        package hello.extended;
        service ExtendedGreeter as CustomServer {
            SayHello,
        }
    }

    #[tokio::test]
    async fn proc_macro_nested_package() {
        let mut server = CustomServer::start_default().await;

        server.setup(
            MockBuilder::when()
                .path_say_hello()
                .then()
                .return_body(|| HelloReply {
                    message: "Nested package works!".into(),
                }),
        );

        let channel = tonic::transport::Channel::from_shared(format!(
            "http://[::1]:{}",
            server.address().port()
        ))
        .unwrap()
        .connect()
        .await
        .unwrap();
        let mut client = ExtendedGreeterClient::new(channel);

        let response = client
            .say_hello(HelloRequest {
                name: "Test".into(),
            })
            .await
            .unwrap();

        assert_eq!("Nested package works!", response.into_inner().message);
    }
}

async fn create() -> (GreeterMockServer, GreeterClient<Channel>) {
    let server = GreeterMockServer::start_default().await;

    let channel =
        tonic::transport::Channel::from_shared(format!("http://[::1]:{}", server.address().port()))
            .unwrap()
            .connect()
            .await
            .unwrap();
    (server, GreeterClient::new(channel))
}
