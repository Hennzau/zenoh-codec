use zenoh_codec::{ZExt, ZStruct, ZStructDecode, ZStructEncode};

#[derive(Default, PartialEq)]
pub struct ZStruct0 {}
impl ZStructEncode for ZStruct0 {
    fn z_len(&self) -> usize {
        0
    }

    fn z_encode(&self, _w: &mut zenoh_codec::ZWriter) -> zenoh_codec::ZResult<()> {
        Ok(())
    }
}

impl<'a> ZStructDecode<'a> for ZStruct0 {
    fn z_decode(_r: &mut zenoh_codec::ZReader<'a>) -> zenoh_codec::ZResult<Self> {
        Ok(ZStruct0 {})
    }
}

impl<'a> ZExt<'a> for ZStruct0 {
    const KIND: zenoh_codec::ZExtKind = zenoh_codec::ZExtKind::ZStruct;
}

#[derive(ZStruct)]
#[zenoh(header = "Z|E|T:3|ID:3=0x3")]
pub struct ZStruct1<'a> {
    pub a: u8,

    #[zenoh(presence = prefixed, size = prefixed)]
    pub b: Option<&'a str>,

    #[zenoh(presence = header(Self::E))]
    pub u: Option<u64>,

    #[zenoh(presence = header(Self::E), size = header(Self::T))]
    pub g: Option<&'a [u8]>,

    #[zenoh(ext = 0x1)]
    pub c: ZStruct0, // Mandatory. Will always be present and sent.
    #[zenoh(ext = 0x2)]
    pub d: Option<ZStruct0>, // Optional. May be absent.
    #[zenoh(ext = 0x3)]
    pub i: Option<ZStruct0>,
    #[zenoh(ext = 0x4)] // Optional. May be absent.
    pub p: Option<ZStruct0>,
    #[zenoh(ext = 0x5, default = Self::DEFAULT_ZSTRUCT0)]
    pub f: ZStruct0, // Not mandatory, has a default value.
}

impl ZStruct1<'_> {
    const DEFAULT_ZSTRUCT0: ZStruct0 = ZStruct0 {};
}

fn main() {}
