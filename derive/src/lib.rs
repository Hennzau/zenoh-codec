pub(crate) mod model;
pub(crate) mod r#struct;

#[proc_macro_derive(ZStruct, attributes(zenoh))]
pub fn derive_zstruct(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    r#struct::derive_zstruct(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
