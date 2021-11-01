<h1 align="center">wiremock grpc</h1>
<div align="center">
 <strong>
   gRPC mocking to test Rust applications.
 </strong>
</div>

<br />


<div align="center">
  <!-- Crates version -->
  <a href="https://crates.io/crates/wiremock-grpc">
    <img src="https://img.shields.io/crates/v/wiremock-grpc.svg?style=flat-square"
    alt="Crates.io version" />
  </a>
  <!-- Downloads -->
  <a href="https://crates.io/crates/wiremock-grpc">
    <img src="https://img.shields.io/crates/d/wiremock-grpc.svg?style=flat-square"
      alt="Download" />
  </a>
  <!-- docs.rs docs -->
  <a href="https://docs.rs/wiremock-grpc">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
      alt="docs.rs docs" />
  </a>
</div>
<br/>

## Example
### Generate Server Code
For each gRPC server you need to generate codes using the [`generate!`] macro.

```rs
mod hello_greeter_mock {
  // hello.Greeter: Is the prefix of all rpc
  // MyHelloServer: Arbitrary name of the generated Server
   wiremock_grpc::generate!("hello.Greeter", MyHelloServer);
}
use hello_greeter_mock::*;
// MyHelloServer, MockBuilder are available to use.
// If multiple servers are generated then use the
// module identifier eg. `hello_greeter_mock::MyHelloServer`
```
### Use it
```rs
#[tokio::test]
async fn handled_when_mock_set() {
    // Server
    let mut server = MyHelloServer::start_default().await;

    server.setup(
        MockBuilder::when()
            .path("/hello.Greeter/SayHello")
            .then()
            .return_status(Code::Ok)
            .return_body(|| HelloReply {
                message: "Hello Mustakim".into(),
            }),
    );

    // Client
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
}
```