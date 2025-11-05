use proc_macro2::TokenStream;

use crate::model::ZenohStruct;

pub fn parse(r#struct: &ZenohStruct) -> syn::Result<TokenStream> {
    Ok(quote::quote! {})
}
