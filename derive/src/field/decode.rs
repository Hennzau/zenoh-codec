use proc_macro2::{Span, TokenStream};
use syn::Ident;

use crate::field::parse::{FieldKind, SizeFlavour, ZField};

pub fn parse_body(field: &ZField, named: bool, flag: TokenStream) -> TokenStream {
    let mut decode_parts = Vec::new();
    let mut result_parts = Vec::new();

    for field in &field.fields {
        let access = &field.access;
        let kind = &field.kind;

        let faccess = if named {
            quote::quote! { #access }
        } else {
            let string = access.to_string();
            let ident = Ident::new(&format!("_field_{}", string), Span::call_site());
            quote::quote! { #ident }
        };

        match kind {
            FieldKind::U8 => {
                decode_parts.push(quote::quote! {
                    let #faccess = <u8 as zenoh_codec::ZField>::z_decode(r)?;
                });
            }
            FieldKind::U16 => {
                decode_parts.push(quote::quote! {
                    let #faccess = <u16 as zenoh_codec::ZField>::z_decode(r)?;
                });
            }
            FieldKind::U32 => {
                decode_parts.push(quote::quote! {
                    let #faccess = <u32 as zenoh_codec::ZField>::z_decode(r)?;
                });
            }
            FieldKind::U64 => {
                decode_parts.push(quote::quote! {
                    let #faccess = <u64 as zenoh_codec::ZField>::z_decode(r)?;
                });
            }
            FieldKind::Usize => {
                decode_parts.push(quote::quote! {
                    let #faccess = <usize as zenoh_codec::ZField>::z_decode(r)?;
                });
            }
            FieldKind::Array(len) => {
                decode_parts.push(quote::quote! {
                    let #faccess = <[u8; #len] as zenoh_codec::ZField>::z_decode(r)?;
                });
            }
            FieldKind::Flag(len) => {
                decode_parts.push(quote::quote! {
                    #flag
                    let #faccess = zenoh_codec::phantom::Flag::<#len>;
                });
            }
            FieldKind::Str(flavour)
            | FieldKind::ZField {
                size_flavour: flavour,
                ..
            } => {
                match flavour {
                    SizeFlavour::Plain => {
                        decode_parts.push(
                        quote::quote! { let #faccess = <usize as zenoh_codec::ZField>::z_decode(r)?; },
                    );
                    }
                    SizeFlavour::Deduced => {
                        decode_parts.push(quote::quote! {
                            let #faccess = <zenoh_codec::ZReader as zenoh_codec::ZReaderExt>::remaining(r);
                        });
                    }
                    _ => {}
                }

                match kind {
                    FieldKind::Str(_) => {
                        decode_parts.push(quote::quote! {
                            let #faccess = <zenoh_codec::ZReader as zenoh_codec::ZReaderExt>::read(r, #faccess)?;
                            let #faccess = core::str::from_utf8(#faccess)
                                .map_err(|_| zenoh_codec::ZCodecError::CouldNotParse)?;
                        });
                    }
                    FieldKind::ZField { path, .. } => {
                        // decode_parts.push(
                        //     quote::quote! { let #faccess = crate::protocol::codec::decode_zid(r, #faccess)?; },
                        // );
                    }
                    _ => unreachable!(),
                };
            }
            _ => {}
        }

        result_parts.push(quote::quote! {
            #access: #faccess
        });
    }

    quote::quote! {
        #(#decode_parts)*

        Ok(Self::ZType { #(#result_parts),* })
    }
}
