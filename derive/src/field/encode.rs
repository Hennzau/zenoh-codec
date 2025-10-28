use proc_macro2::TokenStream;

use crate::field::parse::{FieldKind, OptionFlavour, SizeFlavour, ZField};

pub fn parse_body(field: &ZField, flag: TokenStream) -> TokenStream {
    let mut encode_parts = Vec::new();

    for field in &field.fields {
        let access = &field.access;
        let kind = &field.kind;

        match kind {
            FieldKind::U8 => {
                encode_parts.push(quote::quote! {
                    <u8 as zenoh_codec::ZField>::z_encode(&(self. #access), w)?;
                });
            }
            FieldKind::U16 | FieldKind::U32 | FieldKind::U64 | FieldKind::Usize => {
                encode_parts.push(quote::quote! {
                    <u64 as zenoh_codec::ZField>::z_encode(&(self. #access as u64), w)?;
                });
            }
            FieldKind::Array(len) => {
                encode_parts.push(quote::quote! {
                    <[u8; #len] as zenoh_codec::ZField>::z_encode(&(self. #access), w)?;
                });
            }
            FieldKind::Flag(_) => {
                encode_parts.push(flag.clone());
            }

            FieldKind::Str(flavour)
            | FieldKind::ZField {
                size_flavour: flavour,
                ..
            }
            | FieldKind::OptionZField {
                option_flavour: OptionFlavour::Flag(flavour),
                ..
            }
            | FieldKind::OptionZField {
                option_flavour: OptionFlavour::Plain(flavour),
                ..
            } => {
                match kind {
                    FieldKind::OptionZField {
                        path,
                        option_flavour,
                    } => match option_flavour {
                        OptionFlavour::Plain(_) => {
                            let mut path = path.path.clone();
                            path.segments.last_mut().unwrap().arguments = syn::PathArguments::None;

                            encode_parts.push(quote::quote! {
                                if let Some(value) = &(self. #access) {
                                    <#path as zenoh_codec::ZField>::z_encode(value, w)?;
                                }
                            });
                        }
                        _ => {}
                    },
                    _ => {}
                }

                match flavour {
                    SizeFlavour::Plain => match kind {
                        FieldKind::Str(_) => {
                            encode_parts.push(quote::quote! {
                                <usize as zenoh_codec::ZField>::z_encode(&(self. #access .as_bytes().len()), w)?;
                            });
                        }
                        FieldKind::ZField { path, .. } => {
                            let mut path = path.path.clone();
                            path.segments.last_mut().unwrap().arguments = syn::PathArguments::None;

                            encode_parts.push(quote::quote! {
                                <usize as zenoh_codec::ZField>::z_encode(&<#path as zenoh_codec::ZField>::z_len(&self. #access), w)?;
                            });
                        }
                        FieldKind::OptionZField { path, .. } => {
                            let mut path = path.path.clone();
                            path.segments.last_mut().unwrap().arguments = syn::PathArguments::None;

                            encode_parts.push(quote::quote! {
                                if let Some(value) = &(self. #access) {
                                    <usize as zenoh_codec::ZField>::z_encode(&<#path as zenoh_codec::ZField>::z_len(value), w)?;
                                }
                            });
                        }
                        _ => {}
                    },
                    _ => {}
                }

                match kind {
                    FieldKind::Str(_) => {
                        encode_parts.push(quote::quote! {
                            <zenoh_codec::ZWriter as zenoh_codec::ZWriterExt>::write(w, self. #access .as_bytes())?;
                        });
                    }
                    FieldKind::ZField { path, .. } => {
                        let mut path = path.path.clone();
                        path.segments.last_mut().unwrap().arguments = syn::PathArguments::None;

                        encode_parts.push(quote::quote! {
                            <#path as zenoh_codec::ZField>::z_encode(&(self. #access), w)?;
                        });
                    }
                    FieldKind::OptionZField { path, .. } => {
                        let mut path = path.path.clone();
                        path.segments.last_mut().unwrap().arguments = syn::PathArguments::None;

                        encode_parts.push(quote::quote! {
                            if let Some(value) = &(self. #access) {
                                <#path as zenoh_codec::ZField>::z_encode(value, w)?;
                            }
                        });
                    }
                    _ => {}
                }
            }
        }
    }

    quote::quote! {
        #(#encode_parts)*
        Ok(())
    }
}
