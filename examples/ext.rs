use zenoh_codec::{ZExt, ZExtKind, ZReaderExt, ZStruct, marker, zextattribute};

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

#[derive(ZStruct, PartialEq, Debug)]
pub struct Msg1<'a> {
    // A header acts like a flag but instead of fulling it from the left to the right, each field can apply a bitmask
    _header: marker::Header,

    #[size(plain)]
    field: &'a str,

    // Declare an extension block. Precise how to encode the presence/non presence
    // of at least one extension inside. (available are flag, header or plain)
    #[option(header = 0b1000_0000)]
    _begin: marker::ExtBlockBegin,

    // Extensions in an ExtBlock should always be an option. Failing to do so will result in
    // a compile error but there is no good error message yet.
    pub ext1: Option<ZExt1<'a>>,
    pub ext2: Option<ZExt2>,

    // You should always mark the end of an ext block.
    _end: marker::ExtBlockEnd,

    // You can have other fields after the ext block. You can even have multiple ext blocks.
    #[size(deduced)]
    payload: &'a [u8],
}

zextattribute!(impl<'a> ZExt1<'a>, Msg1<'a>, 0x1, true);
zextattribute!(impl<'a> ZExt2, Msg1<'a>, 0x2, true);

fn main() {
    assert_eq!(ZExt1::KIND, ZExtKind::ZStruct);
    assert_eq!(ZExt2::KIND, ZExtKind::U64);
    assert_eq!(ZExt3::KIND, ZExtKind::Unit);
    assert_eq!(ZExt4::KIND, ZExtKind::ZStruct);

    let x = Msg1 {
        _header: marker::Header,
        field: "hello",
        _begin: marker::ExtBlockBegin,
        ext1: Some(ZExt1 {
            sn: 42,
            qos: 1,
            keyexpr: "/foo/bar",
        }),
        ext2: Some(ZExt2 { sn: 7 }),
        _end: marker::ExtBlockEnd,
        payload: &[1, 2, 3, 4],
    };

    let mut data = [0u8; 128];
    let mut writer = &mut data.as_mut_slice();

    let len = <Msg1 as ZStruct>::z_len(&x);
    <Msg1 as ZStruct>::z_encode(&x, &mut writer).unwrap();

    let mut reader = data.as_slice();
    let decoded = <Msg1 as ZStruct>::z_decode(&mut reader.sub(len).unwrap()).unwrap();

    assert_eq!(x, decoded);
}
