use proc_macro2::TokenStream;

use crate::r#struct::parse::{
    ZPresenceFlavour, ZSizeFlavour, ZStruct, ZStructAttribute, ZStructFieldKind,
};

pub fn parse_body(r#struct: &ZStruct, flag: TokenStream) -> TokenStream {
    let mut len_parts = Vec::new();

    for field in &r#struct.0 {
        let access = &field.access;
        let kind = &field.kind;

        match kind {
            ZStructFieldKind::Flag(len) => {
                let bytes = (len / 8) as usize;
                if bytes == 1 {
                    len_parts.push(quote::quote! {
                        #bytes
                    });
                } else {
                    len_parts.push(quote::quote! {
                        <_ as zenoh_codec::ZStruct>::z_len(&flag)
                    });
                }
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
                    len_parts.push(quote::quote! { 1 });
                }

                if size {
                    if presence {
                        len_parts.push(quote::quote! {
                            if self.#access.is_some() {
                                <usize as zenoh_codec::ZStruct>::z_len(&< #ty as zenoh_codec::ZStruct>::z_len(&self.#access))
                            } else {
                                0
                            }
                        });
                    } else {
                        len_parts.push(quote::quote! {
                            <usize as zenoh_codec::ZStruct>::z_len(&< #ty as zenoh_codec::ZStruct>::z_len(&self.#access))
                        });
                    }
                }

                len_parts.push(quote::quote! {
                    < #ty as zenoh_codec::ZStruct>::z_len(&self.#access)
                });
            }
        }
    }

    let len_body = len_parts
        .into_iter()
        .reduce(|acc, expr| quote::quote! { #acc + #expr })
        .expect("at least one field must be present");

    quote::quote! {
        #flag

        #len_body
    }
}
