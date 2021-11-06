//! Proto generate client and server code
//! using [`tonic_build`]
mod hello;
pub use hello::*;

#[cfg(test)]
mod tests {
    #[test]
    fn generate_proto_client_server() {
        let destination = std::env::current_dir().unwrap().join("src");
        std::env::set_var("OUT_DIR", &destination);
        let destination_path = destination.join("hello.proto");
        tonic_build::compile_protos(destination_path).expect("Unable to generate the code");
    }
}
