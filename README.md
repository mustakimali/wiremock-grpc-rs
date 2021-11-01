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

```rust
mod wiremock_gen {
    // hello.Greeter: is the prefix of all rpc,
    // MyMockServer: name of the generated Server,
    wiremock_grpc::generate!("hello.Greeter", MyMockServer);
}
use wiremock_gen::*;  // this imports generated
use wiremock_grpc::*; // this imports MockBuilder
```

### Use it
```rust
#[tokio::test]
async fn default() {
    // Server (MyMockServer is generated above)
    let mut server = MyMockServer::start_default().await;

    server.setup(
        MockBuilder::when()
            //    👇 RPC prefix
            .path("/hello.Greeter/SayHello")
            .then()
            .return_status(Code::Ok)
            .return_body(|| HelloReply {
                message: "Hello Mustakim".into(),
            }),
    );

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
}
```