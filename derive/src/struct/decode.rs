use proc_macro2::{Span, TokenStream};
use syn::Ident;

use crate::r#struct::parse::{
    ZFieldKind, ZPresenceFlavour, ZSizeFlavour, ZStruct, ZStructFlavour, ZStructKind,
};

pub fn parse_body(r#struct: &ZStruct, flag: TokenStream) -> TokenStream {
    let mut dec = Vec::new();
    let mut declaration = Vec::new();

    for field in &r#struct.0 {
        let access = &field.access;
        let kind = &field.kind;

        declaration.push(quote::quote! {
            #access
        });

        match kind {
            ZFieldKind::Flag => {
                dec.push(quote::quote! {
                    #flag
                    let #access = zenoh_codec::marker::Flag;
                });
            }
            ZFieldKind::Header => {
                dec.push(quote::quote! {
                    let #access = zenoh_codec::marker::Header;
                });
            }
            ZFieldKind::ZExtBlock { flavour, exts } => {
                dec.push(quote::quote! {
                    let #access = zenoh_codec::marker::ExtBlockBegin;
                });

                let paccess = Ident::new(&format!("presence_{}", access), Span::call_site());
                if matches!(flavour, ZPresenceFlavour::Plain) {
                    dec.push(quote::quote! {
                        let mut #paccess: bool = <u8 as zenoh_codec::ZStruct>::z_decode(r)? != 0;
                    });
                }

                let mut body = Vec::new();

                for ext in exts {
                    let access = &ext.access;
                    let ty = &ext.ty;
                    dec.push(quote::quote! {
                        let mut #access: Option<#ty> = None;
                    });

                    declaration.push(quote::quote! {
                        #access
                    });

                    body.push(quote::quote! {
                        < #ty as zenoh_codec::ZExtAttribute<Self>>::ID => {
                            #access = Some(< #ty as zenoh_codec::ZExtAttribute<Self>>::z_decode(r)?);
                        }
                    });
                }

                dec.push(quote::quote! {
                    while #paccess {
                        let (id, kind, mandatory, more) = zenoh_codec::decode_ext_header(r)?;
                        #paccess = more;

                        match id {
                            #(#body),*,
                            _ => {
                                if mandatory {
                                    return Err(zenoh_codec::ZCodecError::MissingMandatoryExtension);
                                }

                                zenoh_codec::ext::skip_ext(r, kind)?;
                            }
                        }
                    }
                });
            }
            ZFieldKind::ZExtBlockEnd => {
                dec.push(quote::quote! {
                    let #access = zenoh_codec::marker::ExtBlockEnd;
                });
            }
            ZFieldKind::ZStruct(ZStructKind { flavour, ty }) => {
                let paccess = Ident::new(&format!("presence_{}", access), Span::call_site());
                let paccess = quote::quote! { #paccess };

                let (p, size) = match flavour {
                    ZStructFlavour::Option { presence, size } => (Some(presence), size),
                    ZStructFlavour::Size(size) => (None, size),
                };

                if matches!(p, Some(ZPresenceFlavour::Plain)) {
                    dec.push(quote::quote! {
                        let #paccess: bool = <u8 as zenoh_codec::ZStruct>::z_decode(r)? != 0;
                    });
                }

                let p_fn = |a: TokenStream, b: TokenStream| if p.is_some() { a } else { b };
                let pa_fn = |tk: &TokenStream| {
                    quote::quote! { let #access = if #paccess { Some(#tk) } else { None }; }
                };
                let decode = quote::quote! { < #ty as zenoh_codec::ZStruct>::z_decode(&mut < zenoh_codec::ZReader as zenoh_codec::ZReaderExt>::sub(r, #access)?)? };
                let decode_size = quote::quote! { < usize as zenoh_codec::ZStruct>::z_decode(r)? };
                let deduce = quote::quote! { < #ty as zenoh_codec::ZStruct>::z_decode(r)? };

                match size {
                    ZSizeFlavour::MaybeEmptyFlag(_) | ZSizeFlavour::NonEmptyFlag(_) => {
                        dec.push(p_fn(
                            pa_fn(&decode),
                            quote::quote! { let #access = #decode; },
                        ));
                    }
                    ZSizeFlavour::Plain => {
                        dec.push(p_fn(
                            pa_fn(&quote::quote! {{
                                let #access = #decode_size;
                                #decode
                            }}),
                            quote::quote! {
                                let #access = #decode_size;
                                let #access = #decode;
                            },
                        ));
                    }
                    ZSizeFlavour::None | ZSizeFlavour::Deduced => {
                        dec.push(p_fn(
                            pa_fn(&deduce),
                            quote::quote! {
                                let #access = #deduce;
                            },
                        ));
                    }
                }
            }
        }
    }

    quote::quote! {
        #(#dec)*

        Ok(Self::ZType { #(#declaration),* })
    }
}
