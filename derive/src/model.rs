use proc_macro2::TokenStream;
use syn::{Generics, Ident, LitStr};

use crate::model::{attribute::ZenohAttribute, ty::ZenohType};

pub mod attribute;
pub mod ty;

pub struct ZenohField {
    pub attr: ZenohAttribute,
    pub ty: ZenohType,
    pub access: TokenStream,
}

impl ZenohField {
    pub fn from_field(field: &syn::Field) -> syn::Result<Self> {
        let attr = ZenohAttribute::from_field(field)?;

        let ident = field
            .ident
            .as_ref()
            .ok_or_else(|| syn::Error::new_spanned(field, "Expected named field"))?;

        let access = quote::quote! { self.#ident };

        let ty = ZenohType::from_type(&field.ty)?;
        ty.check_attribute(&attr)?;

        Ok(Self { attr, access, ty })
    }
}

pub struct HeaderDeclaration {
    pub expr: LitStr,
}

pub struct ZenohStruct {
    pub ident: Ident,
    pub generics: Generics,
    pub header: Option<HeaderDeclaration>,
    pub fields: Vec<ZenohField>,
}

impl ZenohStruct {
    pub fn from_derive_input(input: &syn::DeriveInput) -> syn::Result<Self> {
        let fields = match &input.data {
            syn::Data::Struct(data_struct) => data_struct
                .fields
                .iter()
                .map(ZenohField::from_field)
                .collect::<syn::Result<Vec<ZenohField>>>()?,
            _ => {
                return Err(syn::Error::new_spanned(
                    input,
                    "ZStruct can only be derived for structs",
                ));
            }
        };

        let mut header = Option::<HeaderDeclaration>::None;

        for attr in &input.attrs {
            if attr.path().is_ident("zenoh") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("header") {
                        let value = meta.value()?;
                        let expr: LitStr = value.parse()?;
                        header.replace(HeaderDeclaration { expr });
                    }

                    Ok(())
                })?;
            }
        }

        Ok(Self {
            ident: input.ident.clone(),
            generics: input.generics.clone(),
            header,
            fields,
        })
    }
}
