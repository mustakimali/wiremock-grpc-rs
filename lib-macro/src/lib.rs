use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{parse_macro_input, Attribute, DeriveInput, Ident, Result};

#[proc_macro_derive(WireMockGrpcServer, attributes(server))]
pub fn derive_helper_attr(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    println!("item: \"{:#?}\"", input);

    proc_macro::TokenStream::from(impl_register(input))
}


fn impl_register(input: DeriveInput) -> TokenStream {
    let name = &input.ident;
    let attrs = &input.attrs;
    // println!("{:#?}", input);

    let attrs = attrs
        .iter()
        .filter(|attr| attr.path.is_ident("attach"))
        .map(|attr| parse_attach_attribute(&attr).expect("parse failed"))
        .map(|dep| {
            let method: Ident = dep.method;
            let dependencies = dep.dependencies.iter().map(|ident: &Ident| {
                quote! {
                    std::any::TypeId::of::<#ident>()
                }
            });
            quote! {
                srv.#method::<#name, _>(Arc::clone(&this), &[ #(#dependencies),* ]);
            }
        });
    
        //let name = quote! { #name };
        //println!("{}", name);

    quote! {
        impl #name {
            fn attach(self) {
                todo!()
                #(#attrs)*
            }
        }
    }
}

fn parse_attach_attribute(attr: &Attribute) -> Result<Dependency> {
    let list: syn::MetaList = attr.parse_args()?;
    // println!("{:#?}", list);
    let ident = list.path.get_ident().expect("expected identifier");
    let method = Ident::new(&format!("{}_order", ident), Span::call_site());
    println!("{:#?}", method);
    let dependencies = list
        .nested
        .into_pairs()
        .map(|pair| pair.into_value())
        .filter_map(|pair| match pair {
            syn::NestedMeta::Meta(meta) => match meta {
                syn::Meta::Path(path) => path.get_ident().cloned(),
                _ => panic!("only path meta supported"),
            },
            _ => panic!("lit not supported"),
        })
        .collect();
    println!("{:#?}", dependencies);

    Ok(Dependency {
        method,
        dependencies,
    })
}
struct Dependency {
    method: Ident,
    dependencies: Vec<Ident>,
}