use proc_macro2::TokenStream;

use crate::model::{
    ZenohField, ZenohStruct,
    attribute::{EmptynessAttribute, HeaderAttribute, PresenceAttribute, SizeAttribute},
    ty::ZenohType,
};

pub fn parse(r#struct: &ZenohStruct) -> syn::Result<TokenStream> {
    let mut dec = Vec::<TokenStream>::new();
    let mut declaration = Vec::<TokenStream>::new();

    if r#struct.header.is_some() {
        dec.push(quote::quote! {
            let header: u8 = <u8 as zenoh_codec::ZStructDecode>::z_decode(r)?;
        });
    }

    for field in &r#struct.fields {
        match field {
            ZenohField::Regular { field } => {
                let access = &field.access;
                let ty = &field.ty;
                let attr = &field.attr;

                declaration.push(quote::quote! {
                    #access
                });

                match &attr.header {
                    HeaderAttribute::Mask(mask) => {
                        dec.push(quote::quote! {
                                let #access = {
                                    let v = header & #mask;
                                    <_ as TryFrom<u8>>::try_from(v >> #mask.trailing_zeros()).map_err(|_| zenoh_codec::ZCodecError::CouldNotParse)?
                                };
                            });
                        continue;
                    }
                    _ => {}
                }

                match ty {
                    ZenohType::U8
                    | ZenohType::U16
                    | ZenohType::U32
                    | ZenohType::U64
                    | ZenohType::USize
                    | ZenohType::ByteArray { .. } => {
                        dec.push(quote::quote! {
                            let #access = < _ as zenoh_codec::ZStructDecode>::z_decode(r)?;
                        });
                    }
                    ZenohType::ByteSlice | ZenohType::Str | ZenohType::ZStruct => {
                        match &attr.size {
                            SizeAttribute::Prefixed => {
                                dec.push(quote::quote! {
                                        let #access = < usize as zenoh_codec::ZStructDecode>::z_decode(r)?;
                                        let #access = < _ as zenoh_codec::ZStructDecode>::z_decode(&mut < zenoh_codec::ZReader as zenoh_codec::ZReaderExt>::sub(r, #access)?)?;
                                    });
                            }
                            SizeAttribute::Header(mask) => {
                                let e: u8 =
                                    matches!(attr.emptyness, EmptynessAttribute::NotEmpty) as u8;
                                dec.push(quote::quote! {
                                        let #access = (((header & #mask) >> #mask.trailing_zeros()) + #e) as usize;
                                        let #access = < _ as zenoh_codec::ZStructDecode>::z_decode(&mut < zenoh_codec::ZReader as zenoh_codec::ZReaderExt>::sub(r, #access)?)?;
                                    });
                            }
                            _ => {
                                dec.push(quote::quote! {
                                    let #access = < _ as zenoh_codec::ZStructDecode>::z_decode(r)?;
                                });
                            }
                        }
                    }
                    ZenohType::Option(_) => {
                        match &attr.presence {
                            PresenceAttribute::Prefixed => {
                                dec.push(quote::quote! {
                                    let #access: bool = <u8 as zenoh_codec::ZStructDecode>::z_decode(r)? != 0;
                                });
                            }
                            PresenceAttribute::Header(mask) => {
                                dec.push(quote::quote! {
                                    let #access: bool = (header & #mask) != 0;
                                });
                            }
                            _ => unreachable!(
                                "Option type must have a presence attribute, this was checked before"
                            ),
                        }

                        match &attr.size {
                            SizeAttribute::Prefixed => {
                                dec.push(quote::quote! {
                                    let #access = if #access {
                                        let #access = < usize as zenoh_codec::ZStructDecode>::z_decode(r)?;
                                        Some(< _ as zenoh_codec::ZStructDecode>::z_decode(&mut < zenoh_codec::ZReader as zenoh_codec::ZReaderExt>::sub(r, #access)?)?)
                                    } else {
                                        None
                                    };
                                });
                            }
                            SizeAttribute::Header(mask) => {
                                let e: u8 =
                                    matches!(attr.emptyness, EmptynessAttribute::NotEmpty) as u8;

                                dec.push(quote::quote! {
                                    let #access = if #access {
                                            let #access = (((header & #mask) >> #mask.trailing_zeros()) + #e) as usize;
                                        Some(< _ as zenoh_codec::ZStructDecode>::z_decode(&mut < zenoh_codec::ZReader as zenoh_codec::ZReaderExt>::sub(r, #access)?)?)
                                    } else {
                                        None
                                    };
                                });
                            }
                            _ => {
                                dec.push(quote::quote! {
                                    let #access = if #access {
                                        Some(< _ as zenoh_codec::ZStructDecode>::z_decode(r)?)
                                    } else {
                                        None
                                    };
                                });
                            }
                        }
                    }
                }
            }
            ZenohField::ExtBlock { .. } => {
                //TODO
            }
        }
    }

    Ok(quote::quote! {
        #(#dec)*

        Ok(Self { #(#declaration),* })
    })
}
