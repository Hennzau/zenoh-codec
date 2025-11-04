use proc_macro2::TokenStream;
use syn::DeriveInput;

use crate::model::ZenohStruct;

pub fn derive_zstruct(input: DeriveInput) -> syn::Result<TokenStream> {
    let _ = ZenohStruct::from_derive_input(&input)?;

    Ok(quote::quote! {})
}
