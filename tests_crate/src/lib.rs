#[cfg(test)]
mod test {
    mod mock_server {
        wiremock_grpc::generate!("", MyServer);
    }
    use mock_server::*;

    #[tokio::test]
    async fn it_works() {
        let server = MyServer::start_default().await;

        assert!(std::net::TcpStream::connect(&server.address()).is_ok())
    }
}