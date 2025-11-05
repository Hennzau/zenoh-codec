use proc_macro2::TokenStream;
use syn::Ident;

use crate::model::ZenohStruct;

pub fn parse(r#struct: &ZenohStruct) -> syn::Result<TokenStream> {
    if let Some(header) = &r#struct.header {
        let ident = &r#struct.ident;
        let generics = &r#struct.generics;
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let content = header.expr.value();

        Ok(quote::quote! {
            impl #impl_generics #ident #ty_generics #where_clause {
                const A: &str = "#ident";
            }
        })
    } else {
        Ok(quote::quote! {})
    }
}
