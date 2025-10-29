use proc_macro2::{Span, TokenStream};
use syn::Ident;

use crate::r#struct::parse::{
    ZPresenceFlavour, ZSizeFlavour, ZStruct, ZStructAttribute, ZStructFieldKind,
};

pub fn parse_body(r#struct: &ZStruct) -> (TokenStream, TokenStream) {
    let mut enc_flag_parts = Vec::new();
    let mut dec_flag_parts = Vec::new();

    let mut flag = Option::<u8>::None;
    let mut flag_type = Option::<Ident>::None;
    let mut shift = 0u8;

    for field in &r#struct.0 {
        let access = &field.access;
        let kind = &field.kind;

        match kind {
            ZStructFieldKind::Flag(len) => {
                flag.replace(*len);
                flag_type.replace(Ident::new(&format!("u{}", len), Span::call_site()));
            }
            ZStructFieldKind::ZStruct { attr, ty } => {
                let (presence, (sized, size, maybe_empty)) = match attr {
                    ZStructAttribute::Option { presence, size } => (
                        *presence == ZPresenceFlavour::Flag,
                        match size {
                            ZSizeFlavour::NonEmptyFlag(size) => (true, *size, false),
                            ZSizeFlavour::MaybeEmptyFlag(size) => (true, *size, true),
                            _ => (false, 0u8, false),
                        },
                    ),
                    ZStructAttribute::Size(size) => (
                        false,
                        match size {
                            ZSizeFlavour::NonEmptyFlag(size) => (true, *size, false),
                            ZSizeFlavour::MaybeEmptyFlag(size) => (true, *size, true),
                            _ => (false, 0u8, false),
                        },
                    ),
                };

                if !presence && !sized {
                    continue;
                }

                if (presence || sized) && flag.is_none() {
                    panic!("Flag field must be defined before any field using flag encoding.");
                }

                let flag = flag.unwrap();

                if presence {
                    enc_flag_parts.push(quote::quote! {
                        if self.#access.is_some() {
                            flag |= 1 << ( #flag - 1 - #shift );
                        }
                    });

                    let access = Ident::new(&format!("presence_{}", access), Span::call_site());

                    dec_flag_parts.push(quote::quote! {
                        let #access = ((flag >> ( #flag - 1 - #shift )) & 1) != 0;
                    });

                    shift += 1;
                }

                if sized {
                    let len = quote::quote! {
                        <usize as zenoh_codec::ZStruct>::z_len(&< #ty as zenoh_codec::ZStruct>::z_len(&self.#access))
                    };

                    let masked = if maybe_empty {
                        quote::quote! {
                            ((#len & ((1usize << #size) - 1)) as u8)
                        }
                    } else {
                        quote::quote! {
                            (((#len - 1) & ((1usize << #size) - 1)) as u8)
                        }
                    };

                    enc_flag_parts.push(quote::quote! {
                        flag |= #masked << ( #flag - #size - #shift );
                    });

                    if maybe_empty {
                        dec_flag_parts.push(quote::quote! {
                            let #access =
                                ((flag >> ( #flag - #size - #shift )) & ((1 << #size) - 1)) as usize;
                        });
                    } else {
                        dec_flag_parts.push(quote::quote! {
                            let #access =
                                (((flag >> ( #flag - #size - #shift )) & ((1 << #size) - 1)) as usize) + 1;
                        });
                    }

                    shift += size;
                }
            }
        }
    }

    if flag.is_none() {
        return (quote::quote! {}, quote::quote! {});
    }

    let flag_type = flag_type.unwrap();

    (
        quote::quote! {
            let mut flag: #flag_type = 0;
            #(#enc_flag_parts)*
            <#flag_type as zenoh_codec::ZStruct>::z_encode(&flag, w)?;
        },
        quote::quote! {
            let flag = <#flag_type as zenoh_codec::ZStruct>::z_decode(r)?;
            #(#dec_flag_parts)*
        },
    )
}
