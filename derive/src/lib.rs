mod field;

#[proc_macro_derive(ZField, attributes(header, flag, size, option))]
pub fn derive_zfield(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    field::derive_zfield(input).into()
}
