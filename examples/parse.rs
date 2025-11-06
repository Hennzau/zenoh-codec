use zenoh_codec::{ZExt, ZStruct};

#[derive(ZExt, Default, PartialEq)]
pub struct ZStruct0 {}

#[derive(ZStruct)]
#[zenoh(header = "Z|E|T:3|ID:3=0x3")]
pub struct ZStruct1<'a> {
    #[zenoh(header = Self::E)]
    pub a: u8,

    #[zenoh(presence = prefixed, size = prefixed)]
    pub b: Option<&'a str>,

    #[zenoh(presence = header(Self::E))]
    pub u: Option<u64>,

    #[zenoh(presence = header(Self::E), size = header(Self::T))]
    pub g: Option<&'a [u8]>,

    #[zenoh(ext = 0x1, default = Self::DISABLED_ZSTRUCT0, mandatory)]
    pub c: ZStruct0,
    #[zenoh(ext = 0x2)]
    pub d: Option<ZStruct0>,
    #[zenoh(ext = 0x3)]
    pub i: Option<ZStruct0>,
    #[zenoh(ext = 0x4)]
    pub p: Option<ZStruct0>,
    #[zenoh(ext = 0x5, default = Self::DEFAULT_ZSTRUCT0)]
    pub f: ZStruct0,
}

impl ZStruct1<'_> {
    const DEFAULT_ZSTRUCT0: ZStruct0 = ZStruct0 {};
    const DISABLED_ZSTRUCT0: ZStruct0 = ZStruct0 {};
}

fn main() {
    const V1: u8 = (31u8) & (252u8);
    let a = (5 << V1.trailing_zeros()) & V1;
    println!("V1: {:08b} a: {:08b}", V1, a);
}
