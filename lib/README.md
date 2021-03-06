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
```

## Notes
* It panics when dropped if there are rules set but no requesta are received.
* Request to route without any rules set will return `Unimplemented` gRPC status.

## Limitations
* You have to pass the service prefix (eg. `hello.Greeter`) or RPC path (eg. `/hello.Greeter/SayHello`) as string. These paths are written as string literal in the generated code using `tonic_build`. I have to figure out how access these string literals from a given type or function of the generated code.
* You are unable to spy the request body send to the mock server or set a mock based on a specific request body. I'm yet to get a solid grip on 🦀 to be able to do this.