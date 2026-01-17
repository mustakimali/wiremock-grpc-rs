use crate::wiremock::grpc_server::{GrpcServer, RuleItem};
use tonic::codegen::http::header::IntoHeaderName;
use tonic::codegen::http::{request, HeaderMap, HeaderValue};

pub trait Then {
    fn return_status(self, status: tonic::Code) -> Self;

    fn return_body<T, F>(self, f: F) -> Self
    where
        F: Fn() -> T,
        T: prost::Message;

    fn return_header<K, V>(self, key: K, value: V) -> Self
    where
        K: IntoHeaderName,
        V: TryInto<HeaderValue>,
        <V as TryInto<HeaderValue>>::Error: std::fmt::Debug;
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
    pub(crate) request_headers: HeaderMap,
    pub(crate) response_headers: HeaderMap,
}

#[derive(Clone)]
pub struct WhenBuilder {
    path: Option<String>,
    headers: HeaderMap,
}
impl WhenBuilder {
    #[deprecated(
        since = "0.4.0",
        note = "Use the type-safe API instead. String-based API is there for backward compatibility but will be removed in the future."
    )]
    pub fn path(&self, p: &str) -> Self {
        Self {
            path: Some(p.into()),
            headers: self.headers.clone(),
        }
    }

    pub fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: IntoHeaderName,
        V: TryInto<HeaderValue>,
        <V as TryInto<HeaderValue>>::Error: std::fmt::Debug,
    {
        self.headers.insert(key, value.try_into().unwrap());
        self
    }

    pub fn then(&self) -> ThenBuilder {
        self.validate();
        ThenBuilder {
            path: self.path.clone().unwrap(),
            status_code: None,
            result: None,
            request_headers: self.headers.clone(),
            response_headers: HeaderMap::new(),
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
    pub(crate) request_headers: HeaderMap,
    pub(crate) response_headers: HeaderMap,
}

impl MockBuilder {
    pub fn given(path: &str) -> Self {
        Self {
            path: path.into(),
            result: None,
            status_code: None,
            request_headers: HeaderMap::new(),
            response_headers: HeaderMap::new(),
        }
    }

    pub fn when() -> WhenBuilder {
        WhenBuilder {
            path: None,
            headers: HeaderMap::new(),
        }
    }

    pub(crate) fn matches<B: http_body::Body + Send + 'static>(
        &self,
        req: &request::Request<B>,
    ) -> bool {
        if self.path != req.uri().path() {
            return false;
        }

        for (key, value) in &self.request_headers {
            if !req.headers().contains_key(key.as_str()) {
                return false;
            }
            let Some(mock_value) = req.headers().get(key.as_str()) else {
                return false;
            };

            if mock_value != value {
                return false;
            }
        }

        true
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
        });
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
        result
            .encode(&mut buf)
            .expect("Unable to encode the message");
        let result = buf.to_vec();

        Self {
            result: Some(result),
            ..self
        }
    }

    fn return_header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: IntoHeaderName,
        V: TryInto<HeaderValue>,
        <V as TryInto<HeaderValue>>::Error: std::fmt::Debug,
    {
        self.response_headers.insert(key, value.try_into().unwrap());
        self
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
        result
            .encode(&mut buf)
            .expect("Unable to encode the message");
        let result = buf.to_vec();

        Self {
            result: Some(result),
            ..self
        }
    }

    fn return_header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: IntoHeaderName,
        V: TryInto<HeaderValue>,
        <V as TryInto<HeaderValue>>::Error: std::fmt::Debug,
    {
        self.response_headers.insert(key, value.try_into().unwrap());
        self
    }
}

#[allow(clippy::from_over_into)]
impl Into<MockBuilder> for ThenBuilder {
    fn into(self) -> MockBuilder {
        MockBuilder {
            path: self.path,
            status_code: self.status_code,
            result: self.result,
            request_headers: self.request_headers,
            response_headers: self.response_headers,
        }
    }
}

impl Mountable for ThenBuilder {
    fn mount(self, s: &mut GrpcServer) {
        let rb: MockBuilder = self.into();
        rb.mount(s);
    }
}
