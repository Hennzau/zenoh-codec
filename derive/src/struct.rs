use proc_macro2::TokenStream;
use syn::DeriveInput;

use crate::model::ZenohStruct;

mod header;

mod encode;
mod len;

pub fn derive_zstruct(input: DeriveInput) -> syn::Result<TokenStream> {
    let r#struct = ZenohStruct::from_derive_input(&input)?;
    let ident = &r#struct.ident;

    let generics = &r#struct.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let header = header::parse(&r#struct)?;

    let len = len::parse(&r#struct)?;
    let encode = encode::parse(&r#struct)?;

    Ok(quote::quote! {
        #header

        impl #impl_generics zenoh_codec::ZStructEncode for #ident #ty_generics #where_clause {
            fn z_len(&self) -> usize {
                #len
            }

            fn z_encode(&self, w: &mut zenoh_codec::ZWriter) -> zenoh_codec::ZResult<()> {
                #encode

                Ok(())
            }
        }
    })
}
