use proc_macro2::Span;
use syn::{Expr, Type, TypeArray, TypeReference};

use crate::model::attribute::{
    DefaultAttribute, ExtAttribute, HeaderAttribute, PresenceAttribute, SizeAttribute,
    ZenohAttribute,
};

pub enum ZenohType {
    U8,
    U16,
    U32,
    U64,
    USize,

    ByteArray { len: Expr },
    ByteSlice,
    Str,

    ZStruct,

    Option(Box<ZenohType>),
}

impl ZenohType {
    pub fn check_attribute(&self, attr: &ZenohAttribute) -> syn::Result<()> {
        let (s, p, h, e, d) = (
            !matches!(attr.size, SizeAttribute::None),
            !matches!(attr.presence, PresenceAttribute::None),
            !matches!(attr.header, HeaderAttribute::None),
            !matches!(attr.ext, ExtAttribute::None),
            !matches!(attr.default, DefaultAttribute::None),
        );

        match self {
            ZenohType::U8 => {
                if s || p || e || d {
                    return Err(syn::Error::new(
                        attr.span,
                        "u8 type does not support size, presence, ext, or default attributes",
                    ));
                }
                Ok(())
            }
            ZenohType::U16 | ZenohType::U32 | ZenohType::U64 | ZenohType::USize => {
                if s || p || h || e || d {
                    return Err(syn::Error::new(
                        attr.span,
                        "u16, u32, u64, and usize types do not support size, presence, header, ext, or default attributes",
                    ));
                }
                Ok(())
            }
            ZenohType::ByteArray { .. } | ZenohType::ByteSlice | ZenohType::Str => {
                if p || h || e || d {
                    return Err(syn::Error::new(
                        attr.span,
                        "string and byte array/slice types do not support presence, header, ext, or default attributes",
                    ));
                }
                if !s {
                    return Err(syn::Error::new(
                        attr.span,
                        "string and byte array/slice types require a size attribute",
                    ));
                }
                Ok(())
            }
            ZenohType::ZStruct => {
                if p || h {
                    return Err(syn::Error::new(
                        attr.span,
                        "ZStruct type does not support presence or header attributes",
                    ));
                }
                if e && !d {
                    return Err(syn::Error::new(
                        attr.span,
                        "structs with ext attribute must also have a default attribute",
                    ));
                }
                Ok(())
            }
            ZenohType::Option(inner_ty) => {
                if h || d {
                    return Err(syn::Error::new(
                        attr.span,
                        "Option type does not support header or default attributes",
                    ));
                }

                if !p {
                    return Err(syn::Error::new(
                        attr.span,
                        "Option type requires a presence attribute",
                    ));
                }

                let attr = ZenohAttribute {
                    size: attr.size.clone(),
                    presence: PresenceAttribute::None,
                    header: HeaderAttribute::None,
                    ext: ExtAttribute::None,
                    default: DefaultAttribute::None,
                    span: attr.span.clone(),
                };

                inner_ty.check_attribute(&attr)
            }
        }
    }

    pub fn from_type(ty: &Type) -> syn::Result<Self> {
        match ty {
            Type::Path(type_path) => {
                if type_path.path.segments.first().unwrap().ident == "Option" {
                    if let syn::PathArguments::AngleBracketed(args) =
                        &type_path.path.segments[0].arguments
                    {
                        if args.args.len() == 1 {
                            if let syn::GenericArgument::Type(inner_ty) = &args.args[0] {
                                let zenoh_type = ZenohType::from_type(inner_ty)?;
                                return Ok(ZenohType::Option(Box::new(zenoh_type)));
                            }
                        }
                    }
                    return Err(syn::Error::new_spanned(
                        ty,
                        "Option must have exactly one type argument",
                    ));
                }

                let ident = &type_path.path.segments.last().unwrap().ident;
                match ident.to_string().as_str() {
                    "u8" => Ok(ZenohType::U8),
                    "u16" => Ok(ZenohType::U16),
                    "u32" => Ok(ZenohType::U32),
                    "u64" => Ok(ZenohType::U64),
                    "usize" => Ok(ZenohType::USize),
                    _ => Ok(ZenohType::ZStruct),
                }
            }
            Type::Reference(TypeReference { elem, .. }) => match &**elem {
                Type::Path(type_path) => {
                    let ident = &type_path.path.segments.last().unwrap().ident;
                    if ident == "str" {
                        Ok(ZenohType::Str)
                    } else {
                        Err(syn::Error::new_spanned(ty, "Unsupported reference type"))
                    }
                }
                Type::Slice(syn::TypeSlice { elem, .. }) => match &**elem {
                    Type::Path(type_path) => {
                        let ident = &type_path.path.segments.last().unwrap().ident;
                        if ident == "u8" {
                            Ok(ZenohType::ByteSlice)
                        } else {
                            Err(syn::Error::new_spanned(
                                ty,
                                "Unsupported slice element type",
                            ))
                        }
                    }
                    _ => Err(syn::Error::new_spanned(
                        ty,
                        "Unsupported slice element type",
                    )),
                },
                _ => Err(syn::Error::new_spanned(ty, "Unsupported reference type")),
            },
            Type::Array(TypeArray { elem, len, .. }) => match &**elem {
                Type::Path(type_path) => {
                    let ident = &type_path.path.segments.last().unwrap().ident;
                    if ident == "u8" {
                        Ok(ZenohType::ByteArray { len: len.clone() })
                    } else {
                        Err(syn::Error::new_spanned(
                            ty,
                            "Unsupported array element type",
                        ))
                    }
                }
                _ => Err(syn::Error::new_spanned(
                    ty,
                    "Unsupported array element type",
                )),
            },
            _ => Err(syn::Error::new_spanned(ty, "Unsupported type")),
        }
    }
}
