use regex::Regex;

/// Returns the gRPC service name from a given source and the location of the impl block
/// `impl<T, B> tonic::codegen::Service<http::Request<B>> for {{#TYPE_NAME}}<T>`
fn parse_svc_name<'a>(f: &'a str, line: usize) -> Option<&'a str> {
    let regex = Regex::new(r#"\s"(/[^"]*)" => \{$"#).unwrap();

    for line in f.split('\n').skip(line) {
        let matches = regex.captures_iter(line);
        for m in matches {
            if let Some(capture) = m.get(1) {
                for part in capture.as_str().split("/").filter(|f| f.len() > 0) {
                    return Some(part);
                }
            }
        }
    }

    None
}

macro_rules! find_type {
    ($t:ty, $s: ident) => {
        use crate::WireMockGrpcServer;

        println!("{}", stringify!($t));
        let r = <$t as Clone>::clone($t);

        #[derive(WireMockGrpcServer)]
        struct $s {
            #[server] field: $t,
        }
    };
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_svc_name_works() {
        let path = std::env::current_dir().unwrap().join("../protogen/src/hello.rs");
        let source = std::fs::read_to_string(path).unwrap();

        let result = parse_svc_name(source.as_str(), 157);

        assert!(result.is_some());
        assert_eq!(Some("hello.Greeter"), result);
    }

    #[derive(Clone)]
    struct MyServer;

    #[test]
    fn test_macro() {
        //find_type!(MyServer, MyServerImpl);
    }
}

