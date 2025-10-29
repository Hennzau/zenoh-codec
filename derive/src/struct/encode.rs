use proc_macro2::TokenStream;

use crate::r#struct::parse::{
    ZPresenceFlavour, ZSizeFlavour, ZStruct, ZStructAttribute, ZStructFieldKind,
};

pub fn parse_body(r#struct: &ZStruct, flag: TokenStream) -> TokenStream {
    let mut encode_parts = Vec::new();

    for field in &r#struct.0 {
        let access = &field.access;
        let kind = &field.kind;

        match kind {
            ZStructFieldKind::Flag(_) => {
                encode_parts.push(flag.clone());
            }
            ZStructFieldKind::ZStruct { attr, ty } => {
                let (presence, size) = match attr {
                    ZStructAttribute::Option { presence, size } => (
                        *presence == ZPresenceFlavour::Plain,
                        *size == ZSizeFlavour::Plain,
                    ),
                    ZStructAttribute::Size(size) => (false, *size == ZSizeFlavour::Plain),
                };

                if presence {
                    encode_parts.push(quote::quote! {
                        <u8 as zenoh_codec::ZStruct>::z_encode(
                            &if self. #access .is_some() { 1u8 } else { 0u8 },
                            w,
                        )?;
                    });
                }

                if size {
                    encode_parts.push(quote::quote! {
                        <usize as zenoh_codec::ZStruct>::z_encode(&< #ty as zenoh_codec::ZStruct>::z_len(&self.#access), w)?;
                    });
                }

                encode_parts.push(quote::quote! {
                    < #ty as zenoh_codec::ZStruct>::z_encode(&self.#access, w)?;
                });
            }
        }
    }

    quote::quote! {
        #(#encode_parts)*
        Ok(())
    }
}
