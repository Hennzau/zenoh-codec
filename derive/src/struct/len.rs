use proc_macro2::TokenStream;

use crate::r#struct::parse::{
    ZPresenceFlavour, ZSizeFlavour, ZStruct, ZStructAttribute, ZStructFieldKind,
};

pub fn parse_body(r#struct: &ZStruct) -> TokenStream {
    let mut len_parts = Vec::new();

    for field in &r#struct.0 {
        let access = &field.access;
        let kind = &field.kind;

        match kind {
            ZStructFieldKind::Flag => {
                len_parts.push(quote::quote! {
                    1usize
                });
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
                    len_parts.push(quote::quote! { 1usize });
                }

                if size {
                    let len = quote::quote! {
                        <usize as zenoh_codec::ZStruct>::z_len(&< #ty as zenoh_codec::ZStruct>::z_len(&self.#access))
                    };

                    if presence {
                        len_parts.push(quote::quote! {
                            if self.#access.is_some() {
                                #len
                            } else {
                                0usize
                            }
                        });
                    } else {
                        len_parts.push(len);
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
        #len_body
    }
}
