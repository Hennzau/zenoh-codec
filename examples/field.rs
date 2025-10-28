use zenoh_codec::{ZField, phantom};

mod inner {
    use zenoh_codec::ZField;

    #[derive(ZField)]
    pub struct Field1<'a> {
        pub sn: u32,
        pub qos: u8,

        pub array: [u8; 3],

        // size deduced from the remaining buffer size
        #[size(deduced)]
        pub keyexpr: &'a str,
    }
}

#[derive(ZField)]
struct Field2<'a> {
    pub sn: u32,
    pub qos: u8,

    // Declare a 8-bit flag to store presence/size bits
    #[flag]
    _flag: phantom::Flag<8>,

    // 3 bits to store the size in the flag
    #[size(flag = 3)]
    pub keyexpr: &'a str,

    // size stored as a plain usize before the field
    #[size(plain)]
    pub field1: inner::Field1<'a>,

    // optional presence stored as 1 bit in the flag and
    // 4 bits to store the size of the field in the flag as well
    #[option(flag, size(flag = 4))]
    pub field2: Option<inner::Field1<'a>>,
}

fn main() {
    let a = 0u32;

    let b: u32 = <u32 as Into<u32>>::into(a);
}
