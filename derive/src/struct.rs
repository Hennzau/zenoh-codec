use proc_macro2::TokenStream;
use syn::DeriveInput;

use crate::model::ZenohStruct;

mod header;

pub fn derive_zstruct(input: DeriveInput) -> syn::Result<TokenStream> {
    let r#struct = ZenohStruct::from_derive_input(&input)?;

    let header = header::parse(&r#struct)?;

    Ok(quote::quote! {
        #header
    })
}
