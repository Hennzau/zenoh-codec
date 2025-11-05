use proc_macro2::TokenStream;

use crate::model::{
    ZenohStruct,
    attribute::{ExtAttribute, HeaderAttribute, PresenceAttribute, SizeAttribute, ZenohAttribute},
    ty::ZenohType,
};

fn fill_ty(
    ty: &ZenohType,
    access: &TokenStream,
    attr: &ZenohAttribute,
    len_parts: &mut Vec<TokenStream>,
) -> syn::Result<()> {
    match ty {
        ZenohType::U8
        | ZenohType::U16
        | ZenohType::U32
        | ZenohType::U64
        | ZenohType::USize
        | ZenohType::ByteArray { .. } => {
            if !matches!(attr.header, HeaderAttribute::None) {
                return Ok(());
            }

            len_parts.push(quote::quote! {
                < _ as zenoh_codec::ZStructEncode>::z_len(&self. #access)
            });
        }
        ZenohType::ByteSlice | ZenohType::Str => {
            if matches!(attr.size, SizeAttribute::Prefixed) {
                len_parts.push(quote::quote! {
                    <usize as zenoh_codec::ZStructEncode>::z_len(&< _ as zenoh_codec::ZStructEncode>::z_len(&self. #access))
                });
            }

            len_parts.push(quote::quote! {
                < _ as zenoh_codec::ZStructEncode>::z_len(&self. #access)
            });
        }
        ZenohType::ZStruct => {
            if matches!(attr.size, SizeAttribute::Prefixed) {
                len_parts.push(quote::quote! {
                    <usize as zenoh_codec::ZStructEncode>::z_len(&< _ as zenoh_codec::ZStructEncode>::z_len(&self. #access))
                });
            }

            if !matches!(attr.ext, ExtAttribute::None) {
                len_parts.push(quote::quote! {
                    zenoh_codec::zext_len::<_>(&self. #access)
                });
            } else {
                len_parts.push(quote::quote! {
                    < _ as zenoh_codec::ZStructEncode>::z_len(&self. #access)
                });
            }
        }
        ZenohType::Option(_) => {
            if matches!(attr.presence, PresenceAttribute::Prefixed) {
                len_parts.push(quote::quote! { 1usize });
            }

            if matches!(attr.size, SizeAttribute::Prefixed) {
                len_parts.push(quote::quote! {
                    if let Some(inner) = &self. #access {
                        <usize as zenoh_codec::ZStructEncode>::z_len(&< _ as zenoh_codec::ZStructEncode>::z_len(inner))
                    } else {
                        0usize
                    }
                });
            }

            if !matches!(attr.ext, ExtAttribute::None) {
                len_parts.push(quote::quote! {
                    if let Some(inner) = &self. #access {
                        zenoh_codec::zext_len::<_>(inner)
                    } else {
                        0usize
                    }
                });
            } else {
                len_parts.push(quote::quote! {
                    <_ as zenoh_codec::ZStructEncode>::z_len(&self. #access)
                })
            }
        }
    }

    Ok(())
}

pub fn parse(r#struct: &ZenohStruct) -> syn::Result<TokenStream> {
    let mut len_parts = Vec::new();

    if r#struct.header.is_some() {
        len_parts.push(quote::quote! { 1usize });
    }

    for field in &r#struct.fields {
        let access = &field.access;
        let ty = &field.ty;
        let attr = &field.attr;

        fill_ty(ty, access, attr, &mut len_parts)?;
    }

    if len_parts.is_empty() {
        len_parts.push(quote::quote! { 0usize });
    }

    let len_body = len_parts
        .into_iter()
        .reduce(|acc, expr| quote::quote! { #acc + #expr })
        .unwrap();

    Ok(quote::quote! {
        #len_body
    })
}
