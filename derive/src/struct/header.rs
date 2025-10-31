use proc_macro2::{Span, TokenStream};
use syn::Ident;

use crate::r#struct::parse::{ZFieldKind, ZPresenceFlavour, ZStruct, ZStructFlavour, ZStructKind};

pub fn parse_body(r#struct: &ZStruct) -> (TokenStream, TokenStream) {
    let mut enc = Vec::new();
    let mut dec = Vec::new();

    let mut header = false;

    for field in &r#struct.0 {
        let access = &field.access;
        let kind = &field.kind;

        match kind {
            ZFieldKind::Flag => {}
            ZFieldKind::Header => {
                header = true;
            }
            ZFieldKind::ZExtBlock {
                flavour: ZPresenceFlavour::Header(expr),
                exts,
            } => {
                if !header {
                    panic!("Header field must be defined before any field using header presence.");
                }

                enc.push(quote::quote! {
                    let mut n_exts = 0usize;
                });

                for ext in exts {
                    let access = &ext.access;
                    enc.push(quote::quote! {
                        if self.#access.is_some() {
                            n_exts += 1;
                        }
                    });
                }

                enc.push(quote::quote! {
                    if n_exts > 0 {
                        header |= #expr;
                    }
                });

                let paccess = Ident::new(&format!("presence_{}", access), Span::call_site());

                dec.push(quote::quote! {
                    let mut #paccess = (header & #expr) != 0;
                });
            }
            ZFieldKind::ZExtBlock { .. } => {}
            ZFieldKind::ZExtBlockEnd => {}
            ZFieldKind::ZStruct(ZStructKind {
                flavour:
                    ZStructFlavour::Option {
                        presence: ZPresenceFlavour::Header(expr),
                        ..
                    },
                ..
            }) => {
                if !header {
                    panic!("Header field must be defined before any field using header presence.");
                }

                let paccess = Ident::new(&format!("presence_{}", access), Span::call_site());

                enc.push(quote::quote! {
                    if self.#access.is_some() {
                        header |= #expr;
                    }
                });

                dec.push(quote::quote! {
                    let #paccess: bool = (header & #expr) != 0;
                });
            }
            ZFieldKind::ZStruct(_) => {}
        }
    }

    if !header {
        return (quote::quote! {}, quote::quote! {});
    }

    (
        quote::quote! {
            let mut header: u8 = 0;
            #(#enc)*
            <u8 as zenoh_codec::ZStruct>::z_encode(&header, w)?;
        },
        quote::quote! {
            let header = <u8 as zenoh_codec::ZStruct>::z_decode(r)?;
            #(#dec)*
        },
    )
}
