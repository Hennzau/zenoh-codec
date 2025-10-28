use proc_macro2::TokenStream;
use syn::{
    Attribute, Data, DataStruct, Expr, Field, Fields, GenericArgument, Lit, LitInt, PathArguments,
    TypePath, meta::ParseNestedMeta,
};

pub enum SizeFlavour {
    Plain,
    Deduced,
    NonEmptyFlag(u8),
    MaybeEmptyFlag(u8),
    None,
}

impl SizeFlavour {
    fn from_meta(meta: ParseNestedMeta, flavour: &mut Option<SizeFlavour>) -> syn::Result<()> {
        if meta.path.is_ident("plain") {
            flavour.replace(SizeFlavour::Plain);
        } else if meta.path.is_ident("deduced") {
            flavour.replace(SizeFlavour::Deduced);
        } else if meta.path.is_ident("flag") {
            let value = meta.value().expect("Expected value for flag flavour");
            let lit: LitInt = value.parse()?;
            let flag_index = lit.base10_parse::<u8>()?;

            flavour.replace(SizeFlavour::NonEmptyFlag(flag_index));
        } else if meta.path.is_ident("eflag") {
            let value = meta.value().expect("Expected value for eflag flavour");
            let lit: LitInt = value.parse()?;
            let flag_index = lit.base10_parse::<u8>()?;

            flavour.replace(SizeFlavour::MaybeEmptyFlag(flag_index));
        } else if meta.path.is_ident("none") {
            flavour.replace(SizeFlavour::None);
        }

        Ok(())
    }

    fn from_attr(attr: &Attribute) -> SizeFlavour {
        let mut flavour = Option::<SizeFlavour>::None;

        attr.parse_nested_meta(|meta| Self::from_meta(meta, &mut flavour))
            .expect("Failed to parse size flavour attribute");

        flavour.expect("Field expected to have a size flavour attribute")
    }
}

pub enum OptionFlavour {
    Flag(SizeFlavour),
    Plain(SizeFlavour),
}

impl OptionFlavour {
    fn from_attr(attr: &Attribute) -> OptionFlavour {
        let mut is_flag = false;
        let mut is_plain = false;
        let mut size_flavour = Option::<SizeFlavour>::None;

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("flag") {
                is_flag = true;
            } else if meta.path.is_ident("plain") {
                is_plain = true;
            } else if meta.path.is_ident("size") {
                meta.parse_nested_meta(|size_meta| {
                    SizeFlavour::from_meta(size_meta, &mut size_flavour)
                })?;
            }

            Ok(())
        })
        .expect("Failed to parse option flavour attribute");

        if is_flag {
            OptionFlavour::Flag(
                size_flavour.expect("OptionZField expected to have a size flavour attribute"),
            )
        } else if is_plain {
            OptionFlavour::Plain(
                size_flavour.expect("OptionZField expected to have a size flavour attribute"),
            )
        } else {
            panic!("OptionZField must have either 'flag' or 'plain' attribute");
        }
    }
}

pub enum FieldKind {
    U8,
    U16,
    U32,
    U64,
    Usize,

    Array(Expr),

    Flag(u8),

    Str(SizeFlavour),

    ZField {
        path: TypePath,
        size_flavour: SizeFlavour,
    },

    OptionZField {
        path: TypePath,
        option_flavour: OptionFlavour,
    },
}
impl FieldKind {
    fn from_field(field: &Field) -> FieldKind {
        let ty = &field.ty;
        let attrs = &field.attrs;

        if let syn::Type::Path(tp) = ty {
            if tp.path.is_ident("u8") {
                return FieldKind::U8;
            } else if tp.path.is_ident("u16") {
                return FieldKind::U16;
            } else if tp.path.is_ident("u32") {
                return FieldKind::U32;
            } else if tp.path.is_ident("u64") {
                return FieldKind::U64;
            } else if tp.path.is_ident("usize") {
                return FieldKind::Usize;
            } else if tp
                .path
                .segments
                .last()
                .expect("The last segment of the path should be present.")
                .ident
                == "Flag"
            {
                let args = &tp.path.segments.last().unwrap().arguments;
                if let PathArguments::AngleBracketed(ab) = args {
                    if ab.args.len() != 1 {
                        panic!("Flag type must have exactly one generic argument");
                    }
                    let first_arg = ab.args.first().unwrap();
                    let len = match first_arg {
                        GenericArgument::Const(c) => match c {
                            Expr::Lit(lit) => match &lit.lit {
                                Lit::Int(li) => {
                                    let li = li
                                        .base10_parse::<u8>()
                                        .expect("Flag type argument must be a u8 integer literal");

                                    if li != 8 && li != 16 && li != 32 && li != 64 {
                                        panic!("Flag type argument must be one of: 8, 16, 32, 64");
                                    }

                                    li
                                }
                                _ => panic!("Flag type argument must be an integer literal"),
                            },
                            _ => panic!("Flag type argument must be a literal expression"),
                        },
                        _ => panic!("Flag type argument must be a constant expression"),
                    };

                    return FieldKind::Flag(len);
                } else {
                    panic!("Flag type must have angle bracketed arguments");
                }
            } else {
                let is_option = tp
                    .path
                    .segments
                    .first()
                    .expect("The first segment of the path should be present.")
                    .ident
                    == "Option";

                let path = if is_option {
                    let args = &tp.path.segments.first().unwrap().arguments;

                    if let PathArguments::AngleBracketed(ab) = args {
                        if ab.args.len() != 1 {
                            panic!("Option type must have exactly one generic argument");
                        }

                        let first_arg = ab.args.first().unwrap();
                        match first_arg {
                            GenericArgument::Type(syn::Type::Path(type_path)) => type_path.clone(),
                            _ => panic!("Option type argument must be a type"),
                        }
                    } else {
                        panic!("Option type must have angle bracketed arguments");
                    }
                } else {
                    tp.clone()
                };

                if is_option {
                    let attr = attrs
                        .iter()
                        .find(|a| a.path().is_ident("option"))
                        .expect("Expected option attribute for OptionZField field");

                    let option_flavour = OptionFlavour::from_attr(attr);

                    return FieldKind::OptionZField {
                        path,
                        option_flavour,
                    };
                } else {
                    let attr = attrs
                        .iter()
                        .find(|a| a.path().is_ident("size"))
                        .expect("Expected size attribute for ZField field");

                    let flavour = SizeFlavour::from_attr(attr);

                    return FieldKind::ZField {
                        path,
                        size_flavour: flavour,
                    };
                }
            }
        } else if let syn::Type::Array(ta) = ty {
            return FieldKind::Array(ta.len.clone());
        } else if let syn::Type::Reference(tr) = ty {
            if tr.lifetime.is_none() {
                panic!("Expected lifetime 'a for reference type");
            }
            if tr.mutability.is_some() {
                panic!("Expected immutable reference type");
            }

            if let syn::Type::Path(tp) = &*tr.elem {
                if tp.path.is_ident("str") {
                    let attr = attrs
                        .iter()
                        .find(|a| a.path().is_ident("size"))
                        .expect("Expected size attribute for str field");

                    let flavour = SizeFlavour::from_attr(attr);
                    return FieldKind::Str(flavour);
                }
            }
        }

        panic!("Unsupported field type");
    }
}

pub struct ParsedField {
    pub kind: FieldKind,
    pub access: TokenStream,
}

pub struct ZField {
    pub fields: Vec<ParsedField>,
}

impl ZField {
    fn from_fields<'a>(fields: impl Iterator<Item = &'a Field>) -> ZField {
        let mut parsed_fields = Vec::<ParsedField>::new();
        let mut is_deduced = false;
        let mut flag = Option::<u8>::None;

        let mut total_flag_bits = 0u8;

        for (i, field) in fields.enumerate() {
            if is_deduced {
                panic!("Deduced size flavour must appear once at the end of the struct");
            }

            let kind = FieldKind::from_field(field);

            match &kind {
                FieldKind::Flag(len) => {
                    if flag.is_some() {
                        panic!("Only one Flag field is supported per struct");
                    }

                    flag = Some(*len);
                }

                FieldKind::ZField {
                    path: _,
                    size_flavour: flavour,
                } => match flavour {
                    SizeFlavour::NonEmptyFlag(size) | SizeFlavour::MaybeEmptyFlag(size) => {
                        if flag.is_none() {
                            panic!("Flag field must be defined before using flag size flavours");
                        }

                        total_flag_bits += *size;
                    }
                    SizeFlavour::Deduced => {
                        is_deduced = true;
                    }
                    _ => {}
                },
                FieldKind::OptionZField {
                    path: _,
                    option_flavour,
                } => {
                    if flag.is_none() {
                        panic!("Flag field must be defined before using an optional ZField");
                    }

                    match option_flavour {
                        OptionFlavour::Flag(flavour) => {
                            total_flag_bits += 1;

                            match flavour {
                                SizeFlavour::NonEmptyFlag(size)
                                | SizeFlavour::MaybeEmptyFlag(size) => {
                                    total_flag_bits += *size;
                                }
                                SizeFlavour::Deduced => {
                                    is_deduced = true;
                                }
                                _ => {}
                            }
                        }
                        OptionFlavour::Plain(flavour) => match flavour {
                            SizeFlavour::NonEmptyFlag(size) | SizeFlavour::MaybeEmptyFlag(size) => {
                                total_flag_bits += *size;
                            }
                            SizeFlavour::Deduced => {
                                is_deduced = true;
                            }
                            _ => {}
                        },
                    }
                }

                FieldKind::Str(flavour) => match flavour {
                    SizeFlavour::NonEmptyFlag(size) | SizeFlavour::MaybeEmptyFlag(size) => {
                        if flag.is_none() {
                            panic!("Flag field must be defined before using flag size flavours");
                        }

                        total_flag_bits += *size;
                    }
                    SizeFlavour::Deduced => {
                        is_deduced = true;
                    }
                    _ => {}
                },
                _ => {}
            }

            let access = match &field.ident {
                Some(ident) => quote::quote! { #ident },
                None => {
                    let index = syn::Index::from(i);
                    quote::quote! { #index }
                }
            };

            parsed_fields.push(ParsedField { kind, access });
        }

        if total_flag_bits > 8 {
            panic!("Total flag bits exceed 8 bits");
        }

        if let Some(flag_size) = flag {
            if total_flag_bits > flag_size {
                panic!("Total flag bits exceed defined flag size");
            }
        }

        ZField {
            fields: parsed_fields,
        }
    }

    pub fn from_data(data: &Data) -> (ZField, bool) {
        match data {
            Data::Struct(DataStruct { fields, .. }) => match fields {
                Fields::Unit => (Self::from_fields([].iter()), false),
                Fields::Named(named) => (Self::from_fields(named.named.iter()), true),
                Fields::Unnamed(unnamed) => (Self::from_fields(unnamed.unnamed.iter()), false),
            },
            _ => panic!("infer_kind only supports structs"),
        }
    }
}
