use proc_macro2::{Span, TokenStream};
use syn::Ident;

use crate::r#struct::parse::{
    ZPresenceFlavour, ZSizeFlavour, ZStruct, ZStructAttribute, ZStructFieldKind,
};

pub fn parse_body(r#struct: &ZStruct, flag: TokenStream) -> TokenStream {
    let mut dec = Vec::new();
    let mut res = Vec::new();

    for field in &r#struct.0 {
        let access = &field.access;
        let kind = &field.kind;

        match kind {
            ZStructFieldKind::Flag => {
                dec.push(quote::quote! {
                    #flag
                    let #access = zenoh_codec::marker::Flag;
                });
            }
            ZStructFieldKind::Header => {
                dec.push(quote::quote! {
                    let #access = zenoh_codec::marker::Header;
                });
            }
            ZStructFieldKind::ZStruct { attr, ty } => match attr {
                ZStructAttribute::Option { presence, size } => {
                    let presence_access =
                        Ident::new(&format!("presence_{}", access), Span::call_site());

                    if matches!(*presence, ZPresenceFlavour::Plain) {
                        dec.push(quote::quote! {
                            let #presence_access: bool = <u8 as zenoh_codec::ZStruct>::z_decode(r)? != 0;
                        });
                    }

                    match size {
                        ZSizeFlavour::MaybeEmptyFlag(_) | ZSizeFlavour::NonEmptyFlag(_) => {
                            dec.push(quote::quote! {
                                let #access = if #presence_access {
                                    Some(#access)
                                } else {
                                    None
                                };
                            });
                        }
                        ZSizeFlavour::Plain => {
                            dec.push(quote::quote! {
                                let #access = if #presence_access {
                                    Some(<usize as zenoh_codec::ZStruct>::z_decode(r)?)
                                } else {
                                    None
                                };
                            });
                        }
                        _ => {}
                    }

                    match size {
                        ZSizeFlavour::Plain
                        | ZSizeFlavour::MaybeEmptyFlag(_)
                        | ZSizeFlavour::NonEmptyFlag(_) => {
                            dec.push(quote::quote! {
                                let #access = match #access {
                                    Some(size) => {
                                        Some(< #ty as zenoh_codec::ZStruct>::z_decode(&mut < zenoh_codec::ZReader as zenoh_codec::ZReaderExt>::sub(r, size)?)?)
                                    },
                                    None => None,
                                };
                            });
                        }
                        ZSizeFlavour::None | ZSizeFlavour::Deduced => {
                            dec.push(quote::quote! {
                                let #access = if #presence_access {
                                    Some(< #ty as zenoh_codec::ZStruct>::z_decode(r)?)
                                } else {
                                    None
                                };
                            });
                        }
                    }
                }
                ZStructAttribute::Size(size) => {
                    if *size == ZSizeFlavour::Plain {
                        dec.push(quote::quote! {
                            let #access = <usize as zenoh_codec::ZStruct>::z_decode(r)?;
                        });
                    }

                    match size {
                        ZSizeFlavour::Plain
                        | ZSizeFlavour::MaybeEmptyFlag(_)
                        | ZSizeFlavour::NonEmptyFlag(_) => {
                            dec.push(quote::quote! {
                                let #access = < #ty as zenoh_codec::ZStruct>::z_decode(&mut < zenoh_codec::ZReader as zenoh_codec::ZReaderExt>::sub(r, #access)?)?;
                            });
                        }
                        ZSizeFlavour::None | ZSizeFlavour::Deduced => {
                            dec.push(quote::quote! {
                                let #access = < #ty as zenoh_codec::ZStruct>::z_decode(r)?;
                            });
                        }
                    }
                }
            },
        }

        res.push(quote::quote! {
            #access
        });
    }

    quote::quote! {
        #(#dec)*

        Ok(Self::ZType { #(#res),* })
    }
}
