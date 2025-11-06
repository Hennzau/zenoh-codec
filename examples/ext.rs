use zenoh_codec::{ZExt, ZExtKind, ZReaderExt, ZStruct, ZStructDecode, ZStructEncode};

// A ZExt is a specialized ZStruct that can be used inside another ZStruct as an extension block.
// The kind of ZExt is determined by its fields:
// - If it has one fixed size field, it is specialized into U64.
// - If it has no fields, it is specialized into Unit.
// - Otherwise, it is specialized into ZStruct.
//
// `ZExt1` is a regular ZStruct, so it will be specialized into ZStruct.
#[derive(ZExt, PartialEq, Debug)]
pub struct ZExt1<'a> {
    pub sn: u32,
    pub qos: u8,

    #[zenoh(size = remain)]
    pub keyexpr: &'a str,
}

#[derive(ZExt, PartialEq, Debug)]
pub struct ZExt2 {
    // One fixed size field to be specialized into U64.
    pub sn: u32,
}

const DEFAULT_ZEXT2: ZExt2 = ZExt2 { sn: 0 };

#[derive(ZExt, PartialEq, Debug)]
pub struct ZExt3 {
    // No fields: specialized into Unit.
}

#[derive(ZExt, PartialEq, Debug)]
pub struct ZExt4<'a> {
    #[zenoh(size = remain)]
    pub data: &'a [u8],
}

// Using `ZExt`'s inside a `ZStruct` requires to use a header with a `Z` bit on the left side of the header.
#[derive(ZStruct, PartialEq, Debug)]
#[zenoh(header = "Z|_:7")]
pub struct Msg1<'a> {
    #[zenoh(size = prefixed)]
    field: &'a str,

    // Each ext field must precise its ext id.
    #[zenoh(ext = 0x1)]
    pub ext1: Option<ZExt1<'a>>,
    // If you don't want to use Option, you can use ZExt directly but you must provide a default value.
    #[zenoh(ext = 0x2, default = DEFAULT_ZEXT2)]
    pub ext2: ZExt2,

    // You can have other fields after the ext block.
    #[zenoh(size = remain)]
    payload: &'a [u8],
}

fn main() {
    assert_eq!(ZExt1::KIND, ZExtKind::ZStruct);
    assert_eq!(ZExt2::KIND, ZExtKind::U64);
    assert_eq!(ZExt3::KIND, ZExtKind::Unit);
    assert_eq!(ZExt4::KIND, ZExtKind::ZStruct);

    let x = Msg1 {
        field: "hello",
        ext1: Some(ZExt1 {
            sn: 42,
            qos: 1,
            keyexpr: "/foo/bar",
        }),
        ext2: ZExt2 { sn: 7 },
        payload: &[1, 2, 3, 4],
    };

    let mut data = [0u8; 128];
    let mut writer = &mut data.as_mut_slice();

    let len = <_ as ZStructEncode>::z_len(&x);
    <_ as ZStructEncode>::z_encode(&x, &mut writer).unwrap();

    let mut reader = data.as_slice();
    let decoded = <_ as ZStructDecode>::z_decode(&mut reader.sub(len).unwrap()).unwrap();

    assert_eq!(x, decoded);
}
