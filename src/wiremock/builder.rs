use crate::wiremock::grpc_server::{GrpcServer, RuleItem};

pub trait Then {
    fn return_status(self, status: tonic::Code) -> Self;

    fn return_body<T, F>(self, f: F) -> Self
    where
        F: Fn() -> T,
        T: prost::Message;
}

pub trait Mountable {
    fn mount(self, s: &mut GrpcServer);
}

/// Builder pattern to set up a mock response for a given request.
#[derive(Debug, Clone)]
pub struct MockBuilder {
    pub(crate) path: String,
    pub(crate) status_code: Option<tonic::Code>,
    pub(crate) result: Option<Vec<u8>>,
}

#[derive(Clone)]
pub struct WhenBuilder {
    path: Option<String>,
}
impl WhenBuilder {
    pub fn path(&self, p: &str) -> Self {
        Self {
            path: Some(p.into()),
        }
    }

    pub fn then(&self) -> ThenBuilder {
        self.validate();
        ThenBuilder {
            path: self.path.clone().unwrap(),
            status_code: None,
            result: None,
        }
    }

    fn validate(&self) {
        self.path.as_ref().expect(
            "You must set one or more condition to match (eg. `.when().path(/* ToDo */).then()`)",
        );
    }
}

#[derive(Clone)]
pub struct ThenBuilder {
    pub(crate) path: String,
    pub(crate) status_code: Option<tonic::Code>,
    pub(crate) result: Option<Vec<u8>>,
}

impl MockBuilder {
    pub fn given(path: &str) -> Self {
        Self {
            path: path.into(),
            result: None,
            status_code: None,
        }
    }

    pub fn when() -> WhenBuilder {
        WhenBuilder { path: None }
    }
}

impl Mountable for MockBuilder {
    fn mount(self, s: &mut GrpcServer) {
        if self.status_code.is_none() && self.result.is_none() {
            panic!("Must set the status code or body before attempting to mount the rule.");
        }

        s.rules.write().unwrap().push(RuleItem {
            invocations_count: 0,
            invocations: Vec::default(),
            rule: self,
        })
    }
}

impl Then for MockBuilder {
    fn return_status(self, status: tonic::Code) -> Self {
        Self {
            status_code: Some(status),
            ..self
        }
    }

    fn return_body<T, F>(self, f: F) -> Self
    where
        F: Fn() -> T,
        T: prost::Message,
    {
        let result = f();
        let mut buf = prost::bytes::BytesMut::new();
        let _ = result
            .encode(&mut buf)
            .expect("Unable to encode the message");
        let result = buf.to_vec();

        Self {
            result: Some(result),
            ..self
        }
    }
}

impl Then for ThenBuilder {
    fn return_status(self, status: tonic::Code) -> Self {
        Self {
            status_code: Some(status),
            ..self
        }
    }

    fn return_body<T, F>(self, f: F) -> Self
    where
        F: Fn() -> T,
        T: prost::Message,
    {
        let result = f();
        let mut buf = prost::bytes::BytesMut::new();
        let _ = result
            .encode(&mut buf)
            .expect("Unable to encode the message");
        let result = buf.to_vec();

        Self {
            result: Some(result),
            ..self
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<MockBuilder> for ThenBuilder {
    fn into(self) -> MockBuilder {
        MockBuilder {
            path: self.path,
            status_code: self.status_code,
            result: self.result,
        }
    }
}

impl Mountable for ThenBuilder {
    fn mount(self, s: &mut GrpcServer) {
        let rb: MockBuilder = self.into();

        rb.mount(s)
    }
}
