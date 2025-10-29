use proc_macro2::TokenStream;

use crate::r#struct::parse::{
    ZPresenceFlavour, ZSizeFlavour, ZStruct, ZStructAttribute, ZStructFieldKind,
};

pub fn parse_body(r#struct: &ZStruct, flag: TokenStream) -> TokenStream {
    let mut enc = Vec::new();

    for field in &r#struct.0 {
        let access = &field.access;
        let kind = &field.kind;

        match kind {
            ZStructFieldKind::Flag => {
                enc.push(flag.clone());
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
        }
    }

    quote::quote! {
        #(#enc)*
        Ok(())
    }
}
