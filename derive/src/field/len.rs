use proc_macro2::TokenStream;

use crate::field::parse::{FieldKind, OptionFlavour, SizeFlavour, ZField};

pub fn parse_body(field: &ZField) -> TokenStream {
    let mut len_parts = Vec::new();

    for field in &field.fields {
        let access = &field.access;
        let kind = &field.kind;

        match kind {
            FieldKind::U8 => {
                len_parts.push(quote::quote! { 1 });
            }
            FieldKind::U16 | FieldKind::U32 | FieldKind::U64 | FieldKind::Usize => {
                len_parts.push(
                    quote::quote! { <u64 as zenoh_codec::ZField>::z_len(&(self.#access as u64)) },
                );
            }
            FieldKind::Array(len) => {
                len_parts.push(quote::quote! { #len });
            }
            FieldKind::Str(flavour) => {
                match flavour {
                    SizeFlavour::Plain => {
                        len_parts.push(quote::quote! {
                            <usize as zenoh_codec::ZField>::z_len(&self.#access.as_bytes().len())
                        });
                    }
                    _ => {}
                }

                len_parts.push(quote::quote! { self.#access.as_bytes().len() });
            }
            FieldKind::ZField { path, size_flavour } => {
                let mut path = path.path.clone();
                path.segments.last_mut().unwrap().arguments = syn::PathArguments::None;

                match size_flavour {
                    SizeFlavour::Plain => {
                        len_parts.push(quote::quote! {
                            <usize as zenoh_codec::ZField>::z_len(&< #path as zenoh_codec::ZField>::z_len(&self.#access))
                        });
                    }
                    _ => {}
                }

                len_parts.push(quote::quote! {
                    < #path as zenoh_codec::ZField>::z_len(&self.#access)
                });
            }
            FieldKind::OptionZField {
                path,
                option_flavour,
            } => {
                let size_flavour = match option_flavour {
                    OptionFlavour::Plain(flavour) => {
                        len_parts.push(quote::quote! {
                            1
                        });
                        flavour
                    }
                    OptionFlavour::Flag(flavour) => flavour,
                };

                let mut path = path.path.clone();
                path.segments.last_mut().unwrap().arguments = syn::PathArguments::None;

                match size_flavour {
                    SizeFlavour::Plain => {
                        len_parts.push(quote::quote! {
                            match &self.#access {
                                Some(v) => {
                                    <usize as zenoh_codec::ZField>::z_len(&< #path as zenoh_codec::ZField>::z_len(v))
                                }
                                None => 0,
                            }
                        });
                    }
                    _ => {}
                }

                len_parts.push(quote::quote! {
                    match &self.#access {
                        Some(v) => < #path as zenoh_codec::ZField>::z_len(v),
                        None => 0,
                    }
                });
            }
            FieldKind::Flag(len) => {
                let bytes = (len / 8) as usize;
                len_parts.push(quote::quote! { #bytes });
            }
        }
    }

    let len_body = len_parts
        .into_iter()
        .reduce(|acc, expr| quote::quote! { #acc + #expr })
        .expect("at least one field must be present");

    quote::quote! {
        #len_body
    }
}
