use zenoh_codec::{ZExt, ZExtKind, ZStruct, zextattribute};

// A ZExt is a specialized ZStruct so it must respect all the rules defined for ZStructs.
// Depending on the fields present in the struct the ZExt will be specialized to one of the
// three kinds defined in ZExtKind.
//
// - Unit: if the struct has no fields.
// - U64: if the struct has only fixed size fields (u16, u32, u64, usize).
// - ZStruct: the rest of the cases.
#[derive(ZExt, PartialEq, Debug)]
pub struct ZExt1<'a> {
    pub sn: u32,
    pub qos: u8,

    #[size(deduced)]
    pub keyexpr: &'a str,
}

#[derive(ZExt, PartialEq, Debug)]
pub struct ZExt2 {
    // Only one fixed size field to be specialized as U64 kind.
    pub sn: u32,
}

#[derive(ZExt, PartialEq, Debug)]
pub struct ZExt3 {
    // No fields to be specialized as Unit kind.
}

#[derive(ZExt, PartialEq, Debug)]
pub struct ZExt4<'a> {
    // A regular ZStruct
    #[size(deduced)]
    pub data: &'a [u8],
}

#[derive(ZStruct)]
pub struct Msg1<'a> {
    // A header acts like a flag but instead of fulling it from the left to the right, each field can apply a bitmask
    // _header: marker::Header,

    // _begin: marker::ExtBlockBegin,
    pub ext1: ZExt1<'a>,
    pub ext2: ZExt2,
    // _end: marker::ExtBlockEnd,
}

zextattribute!(impl<'a> ZExt1<'a>, Msg1<'a>, 0x1, true);
zextattribute!(impl<'a> ZExt2, Msg1<'a>, 0x2, true);

fn main() {
    assert_eq!(ZExt1::KIND, ZExtKind::ZStruct);
    assert_eq!(ZExt2::KIND, ZExtKind::U64);
    assert_eq!(ZExt3::KIND, ZExtKind::Unit);
    assert_eq!(ZExt4::KIND, ZExtKind::ZStruct);
}
