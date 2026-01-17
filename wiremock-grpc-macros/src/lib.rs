use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    braced,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Ident, Result, Token,
};

/// Generates a complete mock gRPC server with type-safe RPC method builders.
///
/// This macro creates:
/// - A mock server struct (`{ServiceName}MockServer` or custom name with `as`)
/// - An extension trait for `WhenBuilder` with `path_{method_name}` methods
///
/// # Syntax
///
/// ```ignore
/// wiremock_grpc::generate_svc! {
///     package hello;
///     service Greeter {
///         SayHello,
///         WeatherInfo,
///     }
/// }
/// ```
///
/// Or with a custom server name:
///
/// ```ignore
/// wiremock_grpc::generate_svc! {
///     package hello;
///     service Greeter as MyMockServer {
///         SayHello,
///         WeatherInfo,
///     }
/// }
/// ```
///
/// # Generated Code
///
/// The macro generates:
/// - `{ServiceName}MockServer` (or custom name) - the mock server struct
/// - `{ServiceName}TypeSafeExt` trait with `path_{method_name}` methods
///
/// # Example
///
/// ```ignore
/// use wiremock_grpc::generate_svc;
///
/// generate_svc! {
///     package hello;
///     service Greeter {
///         SayHello,
///         WeatherInfo,
///     }
/// }
///
/// #[tokio::test]
/// async fn test_grpc() {
///     let mut server = GreeterMockServer::start_default().await;
///
///     server.setup(
///         MockBuilder::when()
///             .path_say_hello()  // type-safe method!
///             .then()
///             .return_body(|| HelloReply { message: "Hi".into() })
///     );
///
///     // ... test client code
/// }
/// ```
#[proc_macro]
pub fn generate_svc(input: TokenStream) -> TokenStream {
    let service_def = syn::parse_macro_input!(input as ServiceDefinition);
    service_def.generate().into()
}

struct ServiceDefinition {
    package: String,
    service_name: Ident,
    server_name: Ident,
    methods: Punctuated<Ident, Token![,]>,
}

impl Parse for ServiceDefinition {
    fn parse(input: ParseStream) -> Result<Self> {
        // package keyword (not used)
        let _package_kw: Ident = input.parse()?;
        if _package_kw != "package" {
            return Err(syn::Error::new(
                _package_kw.span(),
                "expected `package` keyword",
            ));
        }

        // parse the package name (x.y.z)
        let first: Ident = input.parse()?;
        let mut package = first.to_string();
        while input.peek(Token![.]) {
            let _dot: Token![.] = input.parse()?;
            let next: Ident = input.parse()?;
            package.push('.');
            package.push_str(&next.to_string());
        }

        let _semi: Token![;] = input.parse()?;

        // next line: service <name> [as <custom name>]
        let _service_kw: Ident = input.parse()?;
        if _service_kw != "service" {
            return Err(syn::Error::new(
                _service_kw.span(),
                "expected `service` keyword",
            ));
        }
        let service_name: Ident = input.parse()?;

        let server_name = if input.peek(Token![as]) {
            let _as: Token![as] = input.parse()?;
            input.parse()?
        } else {
            format_ident!("{}MockServer", service_name)
        };

        let content;
        braced!(content in input);

        let methods = content.parse_terminated(Ident::parse, Token![,])?;

        Ok(ServiceDefinition {
            package,
            service_name,
            server_name,
            methods,
        })
    }
}

impl ServiceDefinition {
    fn generate(&self) -> TokenStream2 {
        let ext_trait = self.generate_ext_trait();
        let mock_server = self.generate_mock_server();

        quote! {
            #ext_trait
            #mock_server
        }
    }

    fn generate_ext_trait(&self) -> TokenStream2 {
        let trait_name = format_ident!("{}TypeSafeExt", self.service_name);
        let package = &self.package;
        let service_name = &self.service_name;

        let method_signatures: Vec<_> = self
            .methods
            .iter()
            .map(|method| {
                let fn_name = format_ident!("path_{}", to_snake_case(&method.to_string()));
                quote! {
                    fn #fn_name(&self) -> Self;
                }
            })
            .collect();

        let method_impls: Vec<_> = self
            .methods
            .iter()
            .map(|method| {
                let fn_name = format_ident!("path_{}", to_snake_case(&method.to_string()));
                let path = format!("/{}.{}/{}", package, service_name, method);
                quote! {
                    fn #fn_name(&self) -> Self {
                        #[expect(deprecated)]
                        self.path(#path)
                    }
                }
            })
            .collect();

        quote! {
            pub trait #trait_name {
                #(#method_signatures)*
            }

            impl #trait_name for wiremock_grpc::WhenBuilder {
                #(#method_impls)*
            }
        }
    }

    fn generate_mock_server(&self) -> TokenStream2 {
        let server_name = &self.server_name;
        let package = &self.package;
        let service_name = &self.service_name;
        let prefix = format!("{}.{}", package, service_name);

        quote! {
            #[derive(Clone)]
            pub struct #server_name(wiremock_grpc::GrpcServer);

            impl ::std::ops::Deref for #server_name {
                type Target = wiremock_grpc::GrpcServer;

                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }

            impl ::std::ops::DerefMut for #server_name {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.0
                }
            }

            impl<B> wiremock_grpc::tonic::codegen::Service<wiremock_grpc::tonic::codegen::http::Request<B>> for #server_name
            where
                B: wiremock_grpc::http_body::Body + Send + 'static,
                B::Error: Into<wiremock_grpc::tonic::codegen::StdError> + Send + 'static,
            {
                type Response = wiremock_grpc::tonic::codegen::http::Response<wiremock_grpc::tonic::body::Body>;
                type Error = ::std::convert::Infallible;
                type Future = wiremock_grpc::tonic::codegen::BoxFuture<Self::Response, Self::Error>;

                fn poll_ready(
                    &mut self,
                    _cx: &mut ::std::task::Context<'_>,
                ) -> ::std::task::Poll<Result<(), Self::Error>> {
                    ::std::task::Poll::Ready(Ok(()))
                }

                fn call(&mut self, req: wiremock_grpc::tonic::codegen::http::Request<B>) -> Self::Future {
                    self.0.handle_request(req)
                }
            }

            impl wiremock_grpc::tonic::server::NamedService for #server_name {
                const NAME: &'static str = #prefix;
            }

            impl #server_name {
                pub async fn start_default() -> Self {
                    let port = wiremock_grpc::GrpcServer::find_unused_port()
                        .await
                        .expect("Unable to find an open port");

                    Self(wiremock_grpc::GrpcServer::new(port)).start_internal().await
                }

                pub async fn start(port: u16) -> Self {
                    Self(wiremock_grpc::GrpcServer::new(port)).start_internal().await
                }

                pub async fn start_with_addr(addr: ::std::net::SocketAddr) -> Self {
                    Self(wiremock_grpc::GrpcServer::with_addr(addr)).start_internal().await
                }

                async fn start_internal(&mut self) -> Self {
                    let address = self.address().clone();
                    let thread = ::tokio::spawn(
                        wiremock_grpc::tonic::transport::Server::builder()
                            .add_service(self.clone())
                            .serve(address),
                    );
                    self._start(thread).await;
                    self.to_owned()
                }
            }
        }
    }
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(ch.to_lowercase().next().unwrap());
        } else {
            result.push(ch);
        }
    }
    result
}
