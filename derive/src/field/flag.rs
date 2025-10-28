use proc_macro2::{Span, TokenStream};
use syn::Ident;

use crate::field::parse::{FieldKind, OptionFlavour, SizeFlavour, ZField};

pub fn parse_body(field: &ZField, named: bool) -> (TokenStream, TokenStream) {
    let mut enc_flag_parts = Vec::new();
    let mut dec_flag_parts = Vec::new();
    let mut shift = 0u8;

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
            FieldKind::Str(flavour)
            | FieldKind::ZField {
                path: _,
                size_flavour: flavour,
            }
            | FieldKind::OptionZField {
                path: _,
                option_flavour: OptionFlavour::Flag(flavour),
            }
            | FieldKind::OptionZField {
                path: _,
                option_flavour: OptionFlavour::Plain(flavour),
            } => {
                match kind {
                    FieldKind::OptionZField {
                        path: _,
                        option_flavour: OptionFlavour::Flag(_),
                    } => {
                        enc_flag_parts.push(quote::quote! {
                            if self.#access.is_some() {
                                flag |= 1 << #shift;
                            }
                        });

                        dec_flag_parts.push(quote::quote! {
                            let is_present = ((flag >> #shift) & 1) != 0;
                        });

                        shift += 1;
                    }
                    _ => {}
                }

                let (flag_size, maybe_empty) = match flavour {
                    SizeFlavour::NonEmptyFlag(size) => (*size, false),
                    SizeFlavour::MaybeEmptyFlag(size) => (*size, true),
                    _ => continue,
                };

                let len = match kind {
                    FieldKind::Str(_) => quote::quote! { self.#access .as_bytes().len() },
                    FieldKind::ZField { path, .. } => {
                        let mut path = path.path.clone();
                        path.segments.last_mut().unwrap().arguments = syn::PathArguments::None;

                        quote::quote! { <usize as zenoh_codec::ZField>::z_len(&< #path as zenoh_codec::ZField>::z_len(&self.#access)) }
                    }
                    FieldKind::OptionZField { path, .. } => {
                        let mut path = path.path.clone();
                        path.segments.last_mut().unwrap().arguments = syn::PathArguments::None;

                        quote::quote! {
                            if let Some(#faccess) = &self.#access {
                                <usize as zenoh_codec::ZField>::z_len(&< #path as zenoh_codec::ZField>::z_len(#faccess))
                            } else {
                                0
                            }
                        }
                    }
                    _ => unreachable!(),
                };

                if maybe_empty {
                    enc_flag_parts.push(quote::quote! {
                        flag |= ((#len as u8) & ((1 << #flag_size) - 1)) << #shift;
                    });

                    dec_flag_parts.push(quote::quote! {
                        let #faccess =
                            ((flag >> #shift) & ((1 << #flag_size) - 1)) as usize;
                    });
                } else {
                    enc_flag_parts.push(quote::quote! {
                        flag |= ((#len as u8 - 1) & ((1 << #flag_size) - 1)) << #shift;
                    });

                    dec_flag_parts.push(quote::quote! {
                        let #faccess =
                            (((flag >> #shift) & ((1 << #flag_size) - 1)) as usize) + 1;
                    });
                }

                shift += flag_size;
            }
            _ => {}
        }
    }

    if enc_flag_parts.is_empty() {
        return (quote::quote! {}, quote::quote! {});
    }

    (
        quote::quote! {
            let mut flag: u8 = 0;
            #(#enc_flag_parts)*
            <u8 as zenoh_codec::ZField>::z_encode(&flag, w)?;
        },
        quote::quote! {
            let flag = <u8 as zenoh_codec::ZField>::z_decode(r)?;
            #(#dec_flag_parts)*
        },
    )
}
