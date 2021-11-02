use crate::{mock_server::RequestItem, MockBuilder, MockGrpcServer};

impl MockGrpcServer {
    pub fn find(&self, r: MockBuilder) -> Option<Vec<RequestItem>> {
        for item in self.rules.read().unwrap().iter() {
            let item = item.read().unwrap();
            if item.rule == r {
                let mut result = Vec::default();
                for i in &item.invocations {
                    result.push(i.clone());
                }
                return Some(result);
            }
        }

        return None;
    }
}

impl PartialEq for MockBuilder {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
            && self.status_code == other.status_code
            && self.result == other.result
    }
}
