use zenoh_codec_derive::ZStruct;

pub struct ZStruct0 {}

#[derive(ZStruct)]
pub struct ZStruct1 {
    #[zenoh(presence = prefixed)]
    pub c: Option<u8>,
}

fn main() {}
