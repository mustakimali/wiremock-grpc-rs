#[allow(dead_code)]

use std::path::PathBuf;

use regex::Regex;

/// Given a type name (of a gRPC server) in string and a starting crate toml file,
/// check all local dependencies and return the service name by locating the source codeco
/// of the type name and parse the service name from the source (using [`parse_svc_name`])
pub(crate) fn find_svc_name_for_type(type_name: String, source_crate_cargo_toml: PathBuf) -> Result<Option<String>, anyhow::Error> {
    let search_for_files = |path: &PathBuf, type_name: String| {
        let mut path = path.clone();
        if path.is_dir() {
            path.pop();
        }

        // search for 
    };
    for dep in parse_dependencies(&source_crate_cargo_toml)? {
        match dep {
            Dependency::Local(name, path) => {
                let path = path.join("Cargo.toml");
            },
            Dependency::Remote(d) => {
                println!("Remote dependency not supported yet, skipping: {}", d);
            }
        }
    }
    
    todo!();
}

/// Returns the gRPC service name from a given source and the location of the impl block
/// `impl<T, B> tonic::codegen::Service<http::Request<B>> for {{#TYPE_NAME}}<T>`
pub(crate) fn parse_svc_name<'a>(f: &'a str, line: usize) -> Option<&'a str> {
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

pub(crate) enum Dependency {
    Remote(String),
    Local(String, PathBuf),
}

pub(crate) fn parse_dependencies(cargo_toml_path: &PathBuf) -> Result<Vec<Dependency>, anyhow::Error> {
    let items = cargo_toml::Manifest::from_path(cargo_toml_path)?.dependencies;
    let mut cargo_toml_dir = cargo_toml_path.clone();
    cargo_toml_dir.pop(); // remove the file name

    let mut deps = Vec::new();
    for (name, dep) in items {
        deps.push(match dep {
            cargo_toml::Dependency::Simple(name) => Dependency::Remote(name),
            cargo_toml::Dependency::Detailed(d) => {
                if let Some(p) = d.path {
                    let dep_path = cargo_toml_dir.join(p);
                    if !dep_path.exists() {
                        log::warn!("Dependency path does not exist: {}", dep_path.display());
                        continue;
                    }
                    
                    println!("Found local dependency: {} -> ({:?})", name, dep_path);
                    Dependency::Local(name, dep_path)
                } else {
                    Dependency::Remote(name)
                }
            }
        });
    }

    Ok(deps)
}

#[allow(unused_macros)]
macro_rules! find_type {
    ($t:ty, $s: ident) => {
        use crate::WireMockGrpcServer;

        println!("{}", stringify!($t));
        let r = <$t as Clone>::clone($t);

        #[derive(WireMockGrpcServer)]
        struct $s {
            #[server]
            field: $t,
        }
    };
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_svc_name_works() {
        let path = std::env::current_dir()
            .unwrap()
            .join("../protogen/src/hello.rs");
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

    #[test]
    fn parse_cargo_dependencies_works() {
        let path = std::env::current_dir().unwrap().join("../test/Cargo.toml");

        let result = parse_dependencies(&path);

        assert!(result.is_ok());
        let deps = result.unwrap();
        assert_eq!(4, deps.len());

        assert_eq!(
            1,
            deps.iter()
                .filter(|f| match f {
                    Dependency::Remote(_) => true,
                    Dependency::Local(_, _) => false,
                })
                .count()
        );

        let local_deps : Vec<(&String, &PathBuf)> = deps
            .iter()
            .map(|d| match d {
                Dependency::Remote(_) => None,
                Dependency::Local(name, path) => Some((name, path)),
            })
            .filter(|d| d.is_some())
            .map(|d| d.unwrap())
            .collect();

        let (name, path) = local_deps[0];

        assert_eq!("wiremock-grpc", name);
        assert!(path.exists());
    }

    #[test]
    fn find_svc_name_for_type_test() {
        let path = std::env::current_dir().unwrap().join("../test/Cargo.toml");
        let result = find_svc_name_for_type("GreeterServer".to_string(), path);

        assert!(result.is_some());
        assert_eq!(Some("hello.Greeter".into()), result);
    }
}
