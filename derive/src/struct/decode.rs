use proc_macro2::{Span, TokenStream};
use syn::Ident;

use crate::r#struct::parse::{
    ZPresenceFlavour, ZSizeFlavour, ZStruct, ZStructAttribute, ZStructFieldKind,
};

pub fn parse_body(r#struct: &ZStruct, flag: TokenStream) -> TokenStream {
    let mut decode_parts = Vec::new();
    let mut result_parts = Vec::new();

    for field in &r#struct.0 {
        let access = &field.access;
        let kind = &field.kind;

        match kind {
            ZStructFieldKind::Flag(len) => {
                let flag_type = Ident::new(&format!("u{}", len), Span::call_site());
                decode_parts.push(quote::quote! {
                    #flag
                    let #access = zenoh_codec::phantom::Flag::<#flag_type>::new();
                });
            }
            ZStructFieldKind::ZStruct { attr, ty } => match attr {
                ZStructAttribute::Option { presence, size } => {
                    let presence_access =
                        Ident::new(&format!("presence_{}", access), Span::call_site());

                    if *presence == ZPresenceFlavour::Plain {
                        decode_parts.push(quote::quote! {
                            let #presence_access: bool = <u8 as zenoh_codec::ZStruct>::z_decode(r)? != 0;
                        });
                    }

                    match size {
                        ZSizeFlavour::MaybeEmptyFlag(_) | ZSizeFlavour::NonEmptyFlag(_) => {
                            decode_parts.push(quote::quote! {
                                let #access = if #presence_access {
                                    Some(#access)
                                } else {
                                    None
                                };
                            });
                        }
                        ZSizeFlavour::Plain => {
                            decode_parts.push(quote::quote! {
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
                            decode_parts.push(quote::quote! {
                                let #access = match #access {
                                    Some(size) => {
                                        Some(< #ty as zenoh_codec::ZStruct>::z_decode(&mut < zenoh_codec::ZReader as zenoh_codec::ZReaderExt>::sub(r, size)?)?)
                                    },
                                    None => None,
                                };
                            });
                        }
                        ZSizeFlavour::None | ZSizeFlavour::Deduced => {
                            decode_parts.push(quote::quote! {
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
                        decode_parts.push(quote::quote! {
                            let #access = <usize as zenoh_codec::ZStruct>::z_decode(r)?;
                        });
                    }

                    match size {
                        ZSizeFlavour::Plain
                        | ZSizeFlavour::MaybeEmptyFlag(_)
                        | ZSizeFlavour::NonEmptyFlag(_) => {
                            decode_parts.push(quote::quote! {
                                let #access = < #ty as zenoh_codec::ZStruct>::z_decode(&mut < zenoh_codec::ZReader as zenoh_codec::ZReaderExt>::sub(r, #access)?)?;
                            });
                        }
                        ZSizeFlavour::None | ZSizeFlavour::Deduced => {
                            decode_parts.push(quote::quote! {
                                let #access = < #ty as zenoh_codec::ZStruct>::z_decode(r)?;
                            });
                        }
                    }
                }
            },
        }

        result_parts.push(quote::quote! {
            #access
        });
    }

    quote::quote! {
        #(#decode_parts)*

        Ok(Self::ZType { #(#result_parts),* })
    }
}
