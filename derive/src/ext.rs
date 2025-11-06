use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::DeriveInput;

use crate::{
    model::{ZenohField, ZenohStruct, ty::ZenohType},
    r#struct::{decode, encode, header, len},
};

mod u64_decode;
mod u64_encode;
mod u64_len;

pub fn derive_zext(input: DeriveInput) -> syn::Result<TokenStream> {
    let r#struct = ZenohStruct::from_derive_input(&input)?;
    let ident = &r#struct.ident;

    let generics = &r#struct.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let kind = infer_kind(&r#struct)?;
    if matches!(kind, InferredKind::U64) {
        let len = u64_len::parse(&r#struct);
        let encode = u64_encode::parse(&r#struct);
        let decode = u64_decode::parse(&r#struct);

        return Ok(quote::quote! {
            impl<'a> zenoh_codec::ZExt<'a> for #ident #ty_generics #where_clause {
                const KIND: zenoh_codec::ZExtKind = #kind;
            }

            impl #impl_generics zenoh_codec::ZStructEncode for #ident #ty_generics #where_clause {
                fn z_len(&self) -> usize {
                    #len
                }

                fn z_encode(&self, w: &mut zenoh_codec::ZWriter) -> zenoh_codec::ZResult<()> {
                    #encode

                    Ok(())
                }
            }

            impl<'a> zenoh_codec::ZStructDecode<'a> for #ident #ty_generics #where_clause {
                fn z_decode(r: &mut zenoh_codec::ZReader<'a>) -> zenoh_codec::ZResult<Self> {
                    #decode
                }
            }
        });
    }

    let header = header::parse(&r#struct)?;

    let len = len::parse(&r#struct)?;
    let encode = encode::parse(&r#struct)?;
    let decode = decode::parse(&r#struct)?;

    Ok(quote::quote! {
        #header

        impl<'a> zenoh_codec::ZExt<'a> for #ident #ty_generics #where_clause {
            const KIND: zenoh_codec::ZExtKind = #kind;
        }

        impl #impl_generics zenoh_codec::ZStructEncode for #ident #ty_generics #where_clause {
            fn z_len(&self) -> usize {
                #len
            }

            fn z_encode(&self, w: &mut zenoh_codec::ZWriter) -> zenoh_codec::ZResult<()> {
                #encode

                Ok(())
            }
        }

        impl<'a> zenoh_codec::ZStructDecode<'a> for #ident #ty_generics #where_clause {
            fn z_decode(r: &mut zenoh_codec::ZReader<'a>) -> zenoh_codec::ZResult<Self> {
                #decode
            }
        }
    })
}

enum InferredKind {
    Unit,
    U64,
    ZStruct,
}

impl ToTokens for InferredKind {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let kind_token = match self {
            InferredKind::Unit => quote::quote! { zenoh_codec::ZExtKind::Unit },
            InferredKind::U64 => quote::quote! { zenoh_codec::ZExtKind::U64 },
            InferredKind::ZStruct => quote::quote! { zenoh_codec::ZExtKind::ZStruct },
        };

        tokens.extend(kind_token);
    }
}

fn infer_kind(ext: &ZenohStruct) -> syn::Result<InferredKind> {
    if ext.fields.is_empty() {
        Ok(InferredKind::Unit)
    } else if ext.fields.len() == 1 {
        let field = &ext.fields.first().unwrap();

        match field {
            ZenohField::ExtBlock { .. } => Err(syn::Error::new(
                Span::call_site(),
                "Cannot infer ZExtKind from only one ext block field",
            )),
            ZenohField::Regular { field } => match field.ty {
                ZenohType::U8
                | ZenohType::U16
                | ZenohType::U32
                | ZenohType::U64
                | ZenohType::USize => Ok(InferredKind::U64),
                _ => Ok(InferredKind::ZStruct),
            },
        }
    } else {
        Ok(InferredKind::ZStruct)
    }
}
