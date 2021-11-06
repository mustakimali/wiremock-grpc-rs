use crate::{grpc_server::RequestItem, GrpcServer, MockBuilder};

impl GrpcServer {
    /// Finds one or more matched requests for a given request builder.
    ///
    /// ## Returns
    /// * [`None`]: when the given [`MockBuilder`] is not registered using the `setup()` function.
    /// * Empty Vector: when no request was made that matches the builder,
    pub fn find(&self, r: &MockBuilder) -> Option<Vec<RequestItem>> {
        for item in self.rules.read().unwrap().iter() {
            if &item.rule == r {
                let mut result = Vec::default();
                for i in &item.invocations {
                    result.push(i.clone());
                }
                return Some(result);
            }
        }

        None
    }

    /// Finds a single matched request for a given criteria
    ///
    /// ## Panics
    /// * No request matching the criteria (eg. No request receieved by the mock server)
    /// * When more than one request matches the criteria (in this case use [`find`])
    /// * When the criteria is inavlid (not registered with the server using the `setup()` function),
    pub fn find_one(&self, r: &MockBuilder) -> RequestItem {
        if let Some(m) = self.find(r) {
            match m.len() {
                0 => panic!("No request maching the given criteria."),
                d if d > 1 => panic!("More then one request matching the criteria."),
                1 => m[0].clone(),
                _ => todo!(),
            }
        } else {
            panic!("The given MockBuilder is not registered with the mock server.");
        }
    }

    /// Returns number of handled requests
    pub fn find_request_count(&self) -> u32 {
        let mut count = 0;
        for item in self.rules.read().unwrap().iter() {
            count += item.invocations_count;
        }
        count
    }

    /// Return number of rules registered with the server
    pub fn rules_len(&self) -> usize {
        self.rules.read().unwrap().iter().len()
    }

    /// Return number of umatched so far
    pub fn rules_unmatched(&self) -> usize {
        self.rules
            .read()
            .unwrap()
            .iter()
            .filter(|f| f.invocations_count == 0)
            .count()
    }
}

impl PartialEq for MockBuilder {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
            && self.status_code == other.status_code
            && self.result == other.result
    }
}
