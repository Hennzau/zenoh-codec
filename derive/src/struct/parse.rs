use proc_macro2::TokenStream;
use syn::{
    AngleBracketedGenericArguments, Attribute, Data, DataStruct, Field, Fields, GenericArgument,
    LitInt, Path, PathArguments, Type, meta::ParseNestedMeta,
};

#[derive(PartialEq)]
pub enum ZSizeFlavour {
    Plain,
    Deduced,
    NonEmptyFlag(u8),
    MaybeEmptyFlag(u8),
    None,
}

impl ZSizeFlavour {
    fn from_meta(meta: &ParseNestedMeta, flavour: &mut Option<ZSizeFlavour>) -> syn::Result<()> {
        if meta.path.is_ident("plain") {
            flavour.replace(ZSizeFlavour::Plain);
        } else if meta.path.is_ident("deduced") {
            flavour.replace(ZSizeFlavour::Deduced);
        } else if meta.path.is_ident("flag") {
            let value = meta.value().expect("Expected value for flag flavour");
            let lit: LitInt = value.parse()?;
            let flag_index = lit.base10_parse::<u8>()?;

            flavour.replace(ZSizeFlavour::NonEmptyFlag(flag_index));
        } else if meta.path.is_ident("eflag") {
            let value = meta.value().expect("Expected value for eflag flavour");
            let lit: LitInt = value.parse()?;
            let flag_index = lit.base10_parse::<u8>()?;

            flavour.replace(ZSizeFlavour::MaybeEmptyFlag(flag_index));
        } else if meta.path.is_ident("none") {
            flavour.replace(ZSizeFlavour::None);
        }

        Ok(())
    }
}

#[derive(PartialEq)]
pub enum ZPresenceFlavour {
    Flag,
    Plain,
}

impl ZPresenceFlavour {
    fn from_meta(
        meta: &ParseNestedMeta,
        flavour: &mut Option<ZPresenceFlavour>,
    ) -> syn::Result<()> {
        if meta.path.is_ident("flag") {
            flavour.replace(ZPresenceFlavour::Flag);
        } else if meta.path.is_ident("plain") {
            flavour.replace(ZPresenceFlavour::Plain);
        }

        Ok(())
    }
}

pub enum ZStructAttribute {
    Option {
        presence: ZPresenceFlavour,
        size: ZSizeFlavour,
    },
    Size(ZSizeFlavour),
}

impl ZStructAttribute {
    fn from_attr(attr: &Attribute) -> ZStructAttribute {
        let mut struct_attr = Option::<ZStructAttribute>::None;

        if attr.path().is_ident("option") {
            let mut presence_flavour = Option::<ZPresenceFlavour>::None;
            let mut size_flavour = Option::<ZSizeFlavour>::None;

            attr.parse_nested_meta(|meta| {
                ZPresenceFlavour::from_meta(&meta, &mut presence_flavour)?;

                if meta.path.is_ident("size") {
                    meta.parse_nested_meta(|size_meta| {
                        ZSizeFlavour::from_meta(&size_meta, &mut size_flavour)
                    })?;
                }

                Ok(())
            })
            .expect("Failed to parse struct attribute");

            struct_attr.replace(ZStructAttribute::Option {
                presence: presence_flavour
                    .expect("Option struct expected to have a presence flavour"),
                size: size_flavour.unwrap_or(ZSizeFlavour::None),
            });
        } else if attr.path().is_ident("size") {
            let mut size_flavour = Option::<ZSizeFlavour>::None;

            attr.parse_nested_meta(|meta| ZSizeFlavour::from_meta(&meta, &mut size_flavour))
                .expect("Failed to parse struct attribute");

            struct_attr.replace(ZStructAttribute::Size(
                size_flavour.expect("Struct expected to have a size flavour"),
            ));
        }

        struct_attr.expect("Struct expected to have either option or size attribute")
    }
}

pub enum ZStructFieldKind {
    Flag(u8),
    ZStruct {
        attr: ZStructAttribute,
        ty: TokenStream,
    },
}

pub struct ZStructField {
    pub kind: ZStructFieldKind,
    pub access: TokenStream,
}

fn remove_lt_from_path(mut path: Path) -> Path {
    match &mut path.segments.last_mut().unwrap().arguments {
        PathArguments::None => path,
        PathArguments::Parenthesized(_) => panic!("Parenthesized arguments are not supported"),
        PathArguments::AngleBracketed(aba) => {
            let mut new_args = AngleBracketedGenericArguments {
                colon2_token: aba.colon2_token,
                lt_token: aba.lt_token,
                args: syn::punctuated::Punctuated::new(),
                gt_token: aba.gt_token,
            };

            for arg in &aba.args {
                match arg {
                    GenericArgument::Type(ty) => match ty {
                        Type::Reference(tr) => {
                            let mut new_tr = tr.clone();
                            new_tr.lifetime = None;
                            new_tr.mutability = None;
                            new_args
                                .args
                                .push(GenericArgument::Type(Type::Reference(new_tr)));
                        }
                        Type::Path(path) => {
                            let new_path = remove_lt_from_path(path.path.clone());

                            new_args
                                .args
                                .push(GenericArgument::Type(Type::Path(syn::TypePath {
                                    qself: None,
                                    path: new_path,
                                })));
                        }
                        _ => {
                            new_args.args.push(arg.clone());
                        }
                    },
                    _ => {}
                }
            }

            path.segments.last_mut().unwrap().arguments = PathArguments::AngleBracketed(new_args);
            path
        }
    }
}

impl ZStructField {
    fn from_field(field: &Field) -> ZStructField {
        let ty = &field.ty;
        let attrs = &field.attrs;
        let access = match &field.ident {
            Some(ident) => quote::quote! { #ident },
            None => {
                panic!("ZStruct fields must be named");
            }
        };

        if let syn::Type::Path(tp) = ty {
            if tp
                .path
                .segments
                .last()
                .expect("The last segment of the path should be present.")
                .ident
                == "Flag"
            {
                match &tp.path.segments.last().unwrap().arguments {
                    PathArguments::AngleBracketed(aba) => {
                        if aba.args.len() != 1 {
                            panic!("Flag type must have exactly one generic argument");
                        }

                        let first_arg = aba.args.first().unwrap();
                        match first_arg {
                            GenericArgument::Type(Type::Path(path)) => {
                                let len = if path.path.is_ident("u8") {
                                    8u8
                                } else if path.path.is_ident("u16") {
                                    16u8
                                } else if path.path.is_ident("u32") {
                                    32u8
                                } else if path.path.is_ident("u64") {
                                    64u8
                                } else {
                                    panic!("Flag type argument must be one of u8, u16, u32, u64");
                                };

                                return ZStructField {
                                    kind: ZStructFieldKind::Flag(len),
                                    access,
                                };
                            }
                            _ => panic!("Flag type argument must be a type"),
                        }
                    }
                    _ => panic!("Flag type must have angle bracketed arguments"),
                }
            }
        }

        let attr = attrs
            .iter()
            .find(|a| a.path().is_ident("option") || a.path().is_ident("size"))
            .and_then(|attr| Some(ZStructAttribute::from_attr(attr)))
            .unwrap_or(ZStructAttribute::Size(ZSizeFlavour::None));

        let ty = match ty {
            Type::Array(ty) => {
                let len = &ty.len;
                quote::quote! {
                    [u8; #len]
                }
            }
            Type::Reference(ty) => {
                let mut ty = ty.clone();
                ty.lifetime = None;
                ty.mutability = None;
                quote::quote! {
                    #ty
                }
            }
            Type::Path(ty) => {
                let path = remove_lt_from_path(ty.path.clone());

                quote::quote! {
                    #path
                }
            }
            _ => panic!("Unsupported field type in ZStruct"),
        };

        return ZStructField {
            kind: ZStructFieldKind::ZStruct { attr, ty },
            access,
        };
    }
}

pub struct ZStruct(pub Vec<ZStructField>);

impl ZStruct {
    fn from_fields<'a>(fields: impl Iterator<Item = &'a Field>) -> ZStruct {
        let mut parsed_fields = Vec::<ZStructField>::new();
        let mut is_deduced = false;
        let mut flag = Option::<u8>::None;
        let mut total_flag_bits = 0u8;

        for field in fields {
            if is_deduced {
                panic!("Deduced size flavour must appear once at the end of the struct");
            }

            let zfield = ZStructField::from_field(field);

            match &zfield.kind {
                ZStructFieldKind::Flag(len) => {
                    if flag.is_some() {
                        panic!("Only one Flag field is supported per struct");
                    }

                    flag = Some(*len);
                }
                ZStructFieldKind::ZStruct { attr, .. } => {
                    match attr {
                        ZStructAttribute::Option {
                            presence: ZPresenceFlavour::Flag,
                            ..
                        } => {
                            if flag.is_none() {
                                panic!(
                                    "Flag field must be defined before using an optional ZStruct"
                                );
                            }

                            total_flag_bits += 1;
                        }
                        _ => {}
                    }

                    match attr {
                        ZStructAttribute::Size(flavour)
                        | ZStructAttribute::Option { size: flavour, .. } => match flavour {
                            ZSizeFlavour::Deduced => {
                                is_deduced = true;
                            }
                            ZSizeFlavour::NonEmptyFlag(size)
                            | ZSizeFlavour::MaybeEmptyFlag(size) => {
                                if flag.is_none() {
                                    panic!(
                                        "Flag field must be defined before using flag size flavours"
                                    );
                                }
                                total_flag_bits += *size;
                            }
                            _ => {}
                        },
                    }
                }
            }

            parsed_fields.push(zfield);
        }

        if let Some(flag_size) = flag {
            if total_flag_bits > flag_size {
                panic!("Total flag bits exceed defined flag size");
            }
        }

        ZStruct(parsed_fields)
    }

    pub fn from_data(data: &Data) -> ZStruct {
        match data {
            Data::Struct(DataStruct { fields, .. }) => match fields {
                Fields::Named(named) => Self::from_fields(named.named.iter()),
                _ => panic!("ZStruct only supports named fields"),
            },
            _ => panic!("infer_kind only supports structs"),
        }
    }
}
