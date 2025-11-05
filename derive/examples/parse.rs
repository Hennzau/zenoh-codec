use zenoh_codec_derive::ZStruct;

pub struct ZStruct0 {}

#[derive(ZStruct)]
#[zenoh(header = "Z|E|T|ID:5=0x3")]
pub struct ZStruct1 {
    #[zenoh(presence = header(Z))]
    pub c: Option<ZStruct0>,
}

fn main() {}
