mod yoyo {
    wiremock_grpc::generate!("", MyServer);
}
use wiremock_grpc::*;
use yoyo::*;

#[tokio::test]
async fn yo() {
    let f = MyServer::start_default().await;
}
