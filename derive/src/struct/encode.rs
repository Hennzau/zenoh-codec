use proc_macro2::TokenStream;

use crate::model::{
    ZenohStruct,
    attribute::{
        DefaultAttribute, EmptynessAttribute, ExtAttribute, HeaderAttribute, PresenceAttribute,
        SizeAttribute,
    },
    ty::ZenohType,
};

pub fn parse(r#struct: &ZenohStruct) -> syn::Result<TokenStream> {
    let mut enc = Vec::<TokenStream>::new();

    if r#struct.header.is_some() {
        enc.push(quote::quote! {
            let mut header: u8 = Self::BASE_HEADER;
        });
    }

    let mut ext_block = false;
    for field in &r#struct.fields {
        let access = &field.access;
        if matches!(field.attr.ext, ExtAttribute::Expr(_)) {
            if !ext_block {
                enc.push(quote::quote! { let mut n_exts = 0usize; });
                ext_block = true;
            }

            match &field.ty {
                ZenohType::Option(_) => {
                    enc.push(quote::quote! {
                        if self. #access .is_some() {
                            n_exts += 1;
                        }
                    });
                }
                ZenohType::ZStruct => match &field.attr.default {
                    DefaultAttribute::Expr(expr) => {
                        enc.push(quote::quote! {
                            if &self. #access  != &#expr {
                                n_exts += 1;
                            }
                        });
                    }
                    DefaultAttribute::None => {
                        enc.push(quote::quote! {
                            n_exts += 1;
                        });
                    }
                },
                _ => {}
            }
        }
    }

    if ext_block {
        enc.push(quote::quote! {
            if n_exts > 0 {
                header |= Self::Z;
            }
        });
    }

    for field in &r#struct.fields {
        let access = &field.access;
        let ty = &field.ty;
        let attr = &field.attr;

        match &attr.header {
            HeaderAttribute::Mask(mask) => {
                enc.push(quote::quote! { header  |= {
                    let v: u8 = self. #access.into();
                    (v << (#mask .trailing_zeros())) & #mask
                }; });
                continue;
            }
            _ => {}
        }

        // Lots of checks have been made in the `ty.rs` file so you can merge lots of cases without worrying
        // about invalid combinations
        match ty {
            ZenohType::U8
            | ZenohType::U16
            | ZenohType::U32
            | ZenohType::U64
            | ZenohType::USize
            | ZenohType::ByteArray { .. } => {
                enc.push(quote::quote! {
                    < _ as zenoh_codec::ZStructEncode>::z_encode(&self. #access, w)?;
                });
            }
            ZenohType::ByteSlice | ZenohType::Str | ZenohType::ZStruct => {
                match &attr.size {
                    SizeAttribute::Prefixed => {
                        enc.push(quote::quote! {
                            <usize as zenoh_codec::ZStructEncode>::z_encode(&< _ as zenoh_codec::ZStructEncode>::z_len(&self. #access), w)?;
                        });
                    }
                    SizeAttribute::Header(mask) => {
                        let e: u8 = matches!(attr.emptyness, EmptynessAttribute::NotEmpty) as u8;
                        enc.push(quote::quote! {
                            header |= {
                                let shift = #mask .trailing_zeros();
                                let len = < _ as zenoh_codec::ZStructEncode>::z_len(&self. #access) as u8;

                                ((len - #e) << shift) & #mask
                            };
                        });
                    }
                    _ => {}
                }

                match &attr.ext {
                    ExtAttribute::Expr(id) => match &attr.default {
                        DefaultAttribute::Expr(expr) => {
                            enc.push(quote::quote! {
                                if &self. #access  != &#expr {
                                    n_exts -= 1;
                                    zenoh_codec::zext_encode::<_, #id, false>(&self. #access, w, n_exts != 0)?;
                                }
                            });
                        }
                        _ => enc.push(quote::quote! {
                            n_exts -= 1;
                            zenoh_codec::zext_encode::<_, #id, true>(&self. #access, w, n_exts != 0)?;
                        }),
                    },
                    _ => {
                        enc.push(quote::quote! {
                            < _ as zenoh_codec::ZStructEncode>::z_encode(&self. #access, w)?;
                        });
                    }
                }
            }
            ZenohType::Option(_) => {
                match &attr.presence {
                    PresenceAttribute::Prefixed => {
                        enc.push(quote::quote! {
                            if self. #access .is_some() {
                                <u8 as zenoh_codec::ZStructEncode>::z_encode(&1u8, w)?;
                            }
                        });
                    }
                    PresenceAttribute::Header(mask) => {
                        enc.push(quote::quote! {
                            if self. #access .is_some() {
                                header |= #mask ;
                            }
                        });
                    }
                    _ => {}
                }

                match &attr.size {
                    SizeAttribute::Prefixed => {
                        enc.push(quote::quote! {
                            if let Some(inner) = &self. #access {
                                <usize as zenoh_codec::ZStructEncode>::z_encode(&< _ as zenoh_codec::ZStructEncode>::z_len(inner), w)?;
                            }
                        });
                    }
                    SizeAttribute::Header(mask) => {
                        let e: u8 = matches!(attr.emptyness, EmptynessAttribute::NotEmpty) as u8;
                        enc.push(quote::quote! {
                            if let Some(inner) = &self. #access {
                                header |= {
                                    let shift = #mask .trailing_zeros();
                                    let len = < _ as zenoh_codec::ZStructEncode>::z_len(inner) as u8;

                                    ((len - #e) << shift) & #mask
                                };
                            }
                        });
                    }
                    _ => {}
                }

                match &attr.ext {
                    ExtAttribute::Expr(id) => {
                        enc.push(quote::quote! {
                            if let Some(inner) = &self. #access {
                                n_exts -= 1;
                                zenoh_codec::zext_encode::<_, #id, false>(inner, w, n_exts != 0)?;
                            }
                        });
                    }
                    _ => {
                        enc.push(quote::quote! {
                            if let Some(inner) = &self. #access {
                                <_ as zenoh_codec::ZStructEncode>::z_encode(inner, w)?;
                            }
                        });
                    }
                }
            }
        }
    }

    Ok(quote::quote! {
        #(#enc)*
    })
}
