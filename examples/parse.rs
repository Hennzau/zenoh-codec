use zenoh_codec::{ZExt, ZStruct, ZStructDecode, ZStructEncode};

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

    #[zenoh(presence = header(Z))]
    pub u: Option<u64>,

    #[zenoh(presence = header(Z), size = header(I))]
    pub g: Option<&'a [u8]>,

    #[zenoh(ext = 0x1)]
    pub c: ZStruct0,
    #[zenoh(ext = 0x2)]
    pub d: Option<ZStruct0>,
    #[zenoh(ext = 0x1)]
    pub i: Option<ZStruct0>,
    #[zenoh(ext = 0x2)]
    pub p: Option<ZStruct0>,
}

fn main() {}
