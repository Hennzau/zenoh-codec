use proc_macro2::Span;
use syn::{Expr, ExprPath, meta::ParseNestedMeta, parenthesized, spanned::Spanned};

#[derive(Clone)]
pub struct ZenohAttribute {
    pub span: Span,

    pub size: SizeAttribute,
    pub emptyness: EmptynessAttribute,
    pub presence: PresenceAttribute,
    pub header: HeaderAttribute,
    pub ext: ExtAttribute,
    pub default: DefaultAttribute,
}

impl Default for ZenohAttribute {
    fn default() -> Self {
        ZenohAttribute {
            span: Span::call_site(),
            size: SizeAttribute::default(),
            emptyness: EmptynessAttribute::default(),
            presence: PresenceAttribute::default(),
            header: HeaderAttribute::default(),
            ext: ExtAttribute::default(),
            default: DefaultAttribute::default(),
        }
    }
}

impl ZenohAttribute {
    pub fn from_field(field: &syn::Field) -> syn::Result<Self> {
        let mut zattr = ZenohAttribute::default();
        zattr.span = field.ident.span();

        for attr in &field.attrs {
            if attr.path().is_ident("zenoh") {
                attr.parse_nested_meta(|meta| {
                    let size = SizeAttribute::from_meta(&meta)?;
                    let emptyness = EmptynessAttribute::from_meta(&meta)?;
                    let presence = PresenceAttribute::from_meta(&meta)?;
                    let header = HeaderAttribute::from_meta(&meta)?;
                    let default = DefaultAttribute::from_meta(&meta)?;
                    let ext = ExtAttribute::from_meta(&meta)?;

                    if !matches!(size, SizeAttribute::None) {
                        zattr.size = size;
                    }
                    if !matches!(emptyness, EmptynessAttribute::NotEmpty) {
                        zattr.emptyness = emptyness;
                    }
                    if !matches!(presence, PresenceAttribute::None) {
                        zattr.presence = presence;
                    }
                    if !matches!(header, HeaderAttribute::None) {
                        zattr.header = header;
                    }
                    if !matches!(ext, ExtAttribute::None) {
                        zattr.ext = ext;
                    }
                    if !matches!(default, DefaultAttribute::None) {
                        zattr.default = default;
                    }

                    Ok(())
                })?;
            }
        }

        Ok(zattr)
    }
}

#[derive(Clone, Default)]
pub enum EmptynessAttribute {
    #[default]
    NotEmpty,
    MaybeEmpty,
}

impl EmptynessAttribute {
    fn from_meta(meta: &ParseNestedMeta) -> syn::Result<Self> {
        if meta.path.is_ident("maybe_empty") {
            return Ok(EmptynessAttribute::MaybeEmpty);
        }

        Ok(EmptynessAttribute::NotEmpty)
    }
}

#[derive(Clone, Default)]
pub enum SizeAttribute {
    #[default]
    None,
    Prefixed,
    Remain,
    Header(ExprPath),
}

impl SizeAttribute {
    fn from_meta(meta: &ParseNestedMeta) -> syn::Result<Self> {
        if meta.path.is_ident("size") {
            let value = meta.value()?;
            let size: syn::Ident = value.parse()?;
            if size == "prefixed" {
                return Ok(SizeAttribute::Prefixed);
            } else if size == "remain" {
                return Ok(SizeAttribute::Remain);
            } else if size == "header" {
                let content;
                parenthesized!(content in value);
                let expr: ExprPath = content.parse()?;
                return Ok(SizeAttribute::Header(expr));
            } else {
                return Err(syn::Error::new_spanned(
                    size,
                    "Invalid size attribute value",
                ));
            }
        }

        Ok(SizeAttribute::None)
    }
}

#[derive(Clone, Default)]
pub enum PresenceAttribute {
    #[default]
    None,
    Prefixed,
    Header(ExprPath),
}

impl PresenceAttribute {
    fn from_meta(meta: &ParseNestedMeta) -> syn::Result<Self> {
        if meta.path.is_ident("presence") {
            let value = meta.value()?;
            let presence: syn::Ident = value.parse()?;
            if presence == "prefixed" {
                return Ok(PresenceAttribute::Prefixed);
            } else if presence == "header" {
                let content;
                parenthesized!(content in value);
                let expr: ExprPath = content.parse()?;
                return Ok(PresenceAttribute::Header(expr));
            } else {
                return Err(syn::Error::new_spanned(
                    presence,
                    "Invalid presence attribute value",
                ));
            }
        }

        Ok(PresenceAttribute::None)
    }
}

#[derive(Clone, Default)]
pub enum HeaderAttribute {
    #[default]
    None,
    Mask(ExprPath),
}

impl HeaderAttribute {
    fn from_meta(meta: &ParseNestedMeta) -> syn::Result<Self> {
        if meta.path.is_ident("header") {
            let expr: ExprPath = meta.value()?.parse()?;
            return Ok(HeaderAttribute::Mask(expr));
        }

        Ok(HeaderAttribute::None)
    }
}

#[derive(Clone, Default)]
pub enum ExtAttribute {
    #[default]
    None,
    Expr(Expr),
}

impl ExtAttribute {
    fn from_meta(meta: &ParseNestedMeta) -> syn::Result<Self> {
        if meta.path.is_ident("ext") {
            let expr: Expr = meta.value()?.parse()?;
            return Ok(ExtAttribute::Expr(expr));
        }

        Ok(ExtAttribute::None)
    }
}

#[derive(Clone, Default)]
pub enum DefaultAttribute {
    #[default]
    None,
    Expr(Expr),
}

impl DefaultAttribute {
    fn from_meta(meta: &ParseNestedMeta) -> syn::Result<Self> {
        if meta.path.is_ident("default") {
            let expr: Expr = meta.value()?.parse()?;
            return Ok(DefaultAttribute::Expr(expr));
        }

        Ok(DefaultAttribute::None)
    }
}
