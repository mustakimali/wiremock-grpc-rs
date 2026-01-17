use wiremock_grpc::MockBuilder;

#[test]
#[should_panic(
    expected = "You must set one or more condition to match (eg. `.when().path(/* ToDo */).then()`)"
)]
fn invalid_mock_builder() {
    MockBuilder::when().then();
}
