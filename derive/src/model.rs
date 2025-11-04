use proc_macro2::TokenStream;
use syn::Ident;

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

pub struct ZenohStruct {
    pub ident: Ident,
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

        Ok(Self {
            ident: input.ident.clone(),
            fields,
        })
    }
}
