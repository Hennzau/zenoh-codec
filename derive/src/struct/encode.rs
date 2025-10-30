use std::intrinsics::unreachable;

use proc_macro2::TokenStream;

use crate::r#struct::parse::{
    ZPresenceFlavour, ZSizeFlavour, ZStruct, ZStructAttribute, ZStructFieldKind,
};

pub fn parse_body(r#struct: &ZStruct, flag: TokenStream) -> TokenStream {
    let mut enc = Vec::new();

    let mut ext_block = Option::<ZPresenceFlavour>::None;

    for field in &r#struct.0 {
        let access = &field.access;
        let kind = &field.kind;

        match kind {
            ZStructFieldKind::Flag => {
                enc.push(flag.clone());
            }
            ZStructFieldKind::ExtBlockBegin(presence) => {
                ext_block.replace(presence.clone());

                enc.push(quote::quote! {
                    let mut n_exts = 0usize;
                });
            }
            ZStructFieldKind::ExtBlockEnd => {
                ext_block.take();
            }
            ZStructFieldKind::ZStruct { attr, ty } => {
                if let Some(p) = ext_block.as_ref() {
                    encode_zext(&mut enc, p, access, attr, ty);
                } else {
                    encode_zstruct(&mut enc, access, attr, ty);
                }
            }
            _ => {}
        }
    }

    quote::quote! {
        #(#enc)*
        Ok(())
    }
}

fn encode_zstruct(
    enc: &mut Vec<TokenStream>,
    access: &TokenStream,
    attr: &ZStructAttribute,
    ty: &TokenStream,
) {
    let (presence, size) = match attr {
        ZStructAttribute::Option { presence, size } => (
            matches!(*presence, ZPresenceFlavour::Plain),
            matches!(*size, ZSizeFlavour::Plain),
        ),
        ZStructAttribute::Size(size) => (false, matches!(*size, ZSizeFlavour::Plain)),
    };

    if presence {
        enc.push(quote::quote! {
            <u8 as zenoh_codec::ZStruct>::z_encode(
                &if self. #access .is_some() { 1u8 } else { 0u8 },
                w,
            )?;
        });
    }

    let len = quote::quote! { <usize as zenoh_codec::ZStruct>::z_encode(&< #ty as zenoh_codec::ZStruct>::z_len(&self. #access), w)?; };
    match (presence, size) {
        (true, true) => {
            enc.push(quote::quote! {
                if self.#access.is_some() {
                    #len
                }
            });
        }
        (false, true) => {
            enc.push(len);
        }
        _ => {}
    }

    enc.push(quote::quote! {
        < #ty as zenoh_codec::ZStruct>::z_encode(&self.#access, w)?;
    });
}

fn encode_zext(
    enc: &mut Vec<TokenStream>,
    presence: &ZPresenceFlavour,
    access: &TokenStream,
    attr: &ZStructAttribute,
    ty: &TokenStream,
) {
    let presence = match presence {
        ZPresenceFlavour::Flag => unreachable(),
        ZPresenceFlavour::Plain => {
            quote::quote! {
                <u8 as crate::ZStruct>::z_encode(&1u8, w)?;
            }
        }
        ZPresenceFlavour::Header(expr) => {
            quote::quote! {
                header
            }
        }
    };

    enc.push(quote::quote! {
        if self. #access .is_some() {
            if n_exts == 0 {

            }

            n_exts += 1;
        }
    });
}
