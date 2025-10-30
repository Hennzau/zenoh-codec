use zenoh_codec::ZExt;

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
    pub sn: u32,
}

#[derive(ZExt, PartialEq, Debug)]
pub struct ZExt3 {}

#[derive(ZExt, PartialEq, Debug)]
pub struct ZExt4<'a> {
    #[size(deduced)]
    pub data: &'a [u8],
}

pub struct Msg1;

fn main() {}
