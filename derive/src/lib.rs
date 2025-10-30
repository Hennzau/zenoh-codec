pub(crate) mod r#ext;
pub(crate) mod r#struct;

#[proc_macro_derive(ZStruct, attributes(size, option))]
pub fn derive_zstruct(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    r#struct::derive_zstruct(input).into()
}

#[proc_macro_derive(ZExt, attributes(size, option))]
pub fn derive_zext(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    r#ext::derive_zext(input).into()
}
