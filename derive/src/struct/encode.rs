use proc_macro2::TokenStream;

use crate::model::{
    ZenohField, ZenohStruct,
    attribute::{
        DefaultAttribute, EmptynessAttribute, ExtAttribute, HeaderAttribute, PresenceAttribute,
        SizeAttribute,
    },
    ty::ZenohType,
};

pub fn parse(r#struct: &ZenohStruct) -> syn::Result<TokenStream> {
    let mut enc = Vec::<TokenStream>::new();
    let mut enc_header = Vec::<TokenStream>::new();

    if r#struct.header.is_some() {
        enc_header.push(quote::quote! {
            let mut header: u8 = Self::BASE_HEADER;
        });
    }

    for field in &r#struct.fields {
        match field {
            ZenohField::Regular { field } => {
                let access = &field.access;
                let ty = &field.ty;
                let attr = &field.attr;

                match &attr.header {
                    HeaderAttribute::Mask(mask) => {
                        enc_header.push(quote::quote! { header  |= {
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
                    | ZenohType::ByteArray { .. }
                    | ZenohType::ByteSlice
                    | ZenohType::Str
                    | ZenohType::ZStruct => {
                        match &attr.size {
                            SizeAttribute::Prefixed => {
                                enc.push(quote::quote! {
                                    <usize as zenoh_codec::ZStructEncode>::z_encode(&< _ as zenoh_codec::ZStructEncode>::z_len(&self. #access), w)?;
                                });
                            }
                            SizeAttribute::Header(mask) => {
                                let e: u8 =
                                    matches!(attr.emptyness, EmptynessAttribute::NotEmpty) as u8;
                                enc_header.push(quote::quote! {
                                    header |= {
                                        let shift = #mask .trailing_zeros();
                                        let len = < _ as zenoh_codec::ZStructEncode>::z_len(&self. #access) as u8;

                                        ((len - #e) << shift) & #mask
                                    };
                                });
                            }
                            _ => {}
                        }

                        enc.push(quote::quote! {
                            < _ as zenoh_codec::ZStructEncode>::z_encode(&self. #access, w)?;
                        });
                    }
                    ZenohType::Option(_) => {
                        match &attr.presence {
                            PresenceAttribute::Prefixed => {
                                enc.push(quote::quote! {
                                    <u8 as zenoh_codec::ZStructEncode>::z_encode(&(self. #access.is_some() as u8), w)?;
                                });
                            }
                            PresenceAttribute::Header(mask) => {
                                enc_header.push(quote::quote! {
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
                                let e: u8 =
                                    matches!(attr.emptyness, EmptynessAttribute::NotEmpty) as u8;
                                enc_header.push(quote::quote! {
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

                        enc.push(quote::quote! {
                            if let Some(inner) = &self. #access {
                                < _ as zenoh_codec::ZStructEncode>::z_encode(inner, w)?;
                            }
                        });
                    }
                }
            }
            ZenohField::ExtBlock { exts } => {
                enc_header.push(quote::quote! {
                    let mut n_exts = 0;
                });

                let mut enc_ext = Vec::<TokenStream>::new();

                for field in exts {
                    let access = &field.access;
                    let ty = &field.ty;
                    let attr = &field.attr;

                    let id = match &attr.ext {
                        ExtAttribute::Expr(id) => id,
                        _ => unreachable!(
                            "ExtBlock fields must have an ext attribute, this should have been caught earlier"
                        ),
                    };

                    match ty {
                        ZenohType::ZStruct => match &attr.default {
                            DefaultAttribute::Expr(expr) => {
                                enc_header.push(quote::quote! {
                                    if &self. #access  != &#expr {
                                        n_exts += 1;
                                    }
                                });

                                enc_ext.push(quote::quote! {
                                    if &self. #access  != &#expr {
                                        zenoh_codec::zext_encode::<_, #id, false>(&self. #access, w, n_exts != 0)?;
                                    }
                                });
                            }
                            _ => {
                                enc_header.push(quote::quote! {
                                    n_exts += 1;
                                });

                                enc_ext.push(quote::quote! {
                                    zenoh_codec::zext_encode::<_, #id, true>(&self. #access, w, n_exts != 0)?;
                                });
                            }
                        },
                        ZenohType::Option(_) => {
                            enc_header.push(quote::quote! {
                                if self. #access .is_some() {
                                    n_exts += 1;
                                }
                            });

                            enc_ext.push(quote::quote! {
                                if let Some(inner) = &self. #access {
                                    zenoh_codec::zext_encode::<_, #id, false>(inner, w, n_exts != 0)?;
                                }
                            });
                        }
                        _ => unreachable!(
                            "Only ZStruct and Option<ZStruct> are allowed in ext blocks, this should have been caught earlier"
                        ),
                    }
                }

                enc.push(quote::quote! {
                    if n_exts > 0 {
                        header |= Self::Z;
                    }

                    #(#enc_ext)*
                });
            }
        }
    }

    if r#struct.header.is_some() {
        enc.insert(
            0,
            quote::quote! {
                <u8 as zenoh_codec::ZStructEncode>::z_encode(&header, w)?;
            },
        );
    }

    Ok(quote::quote! {
        #(#enc_header)*

        #(#enc)*
    })
}
