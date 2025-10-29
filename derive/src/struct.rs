use proc_macro2::TokenStream;
use syn::DeriveInput;

use crate::r#struct::parse::ZStruct;

mod parse;

mod decode;
mod encode;
mod flag;
mod len;

pub fn derive_zstruct(input: DeriveInput) -> TokenStream {
    let ident = &input.ident;
    let data = &input.data;

    let mut lt_params = false;
    for generic in input.generics.params.iter() {
        match generic {
            syn::GenericParam::Lifetime(_) => {
                if lt_params {
                    panic!("ZStruct can only have one lifetime parameter.");
                }

                lt_params = true;
            }
            _ => {
                panic!("ZStruct can only have a lifetime parameter.");
            }
        }
    }

    let (ty_elided, ty_lt) = if lt_params {
        (quote::quote! { <'_> }, quote::quote! { <'a> })
    } else {
        (quote::quote! {}, quote::quote! {})
    };

    let r#struct = ZStruct::from_data(data);

    let (flag, flag_enc, flag_dec) = flag::parse_body(&r#struct);

    let len_body = len::parse_body(&r#struct, flag);
    let encode_body = encode::parse_body(&r#struct, flag_enc);
    let decode_body = decode::parse_body(&r#struct, flag_dec);

    quote::quote! {
        impl zenoh_codec::ZStruct for #ident #ty_elided {
            fn z_len(&self) -> usize {
                #len_body
            }

            fn z_encode(&self, w: &mut zenoh_codec::ZWriter) -> zenoh_codec::ZResult<()> {
                #encode_body
            }

            type ZType<'a> = #ident #ty_lt;

            fn z_decode<'a>(r: &mut zenoh_codec::ZReader<'a>) -> zenoh_codec::ZResult<Self::ZType<'a>> {
                #decode_body
            }
        }
    }
}
