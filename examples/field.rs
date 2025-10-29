use zenoh_codec::{ZStruct, phantom};

mod inner {
    use zenoh_codec::ZStruct;

    #[derive(ZStruct)]
    pub struct Field1<'a> {
        // Fields with fixed sized should not mention a size attribute. If nothing
        // is specified the size flavour is `none`. Which means that when decoding
        // the field should not rely on any size information.
        pub sn: u32,

        // Just to illustrate that size(none) is equivalent to not specifying anything
        #[size(none)]
        pub qos: u8,

        // A fixed sized array of 3 bytes. You don't need to specify any size attribute here.
        pub array: [u8; 3],

        // An optional field should always specify its option flavour.
        pub opt: Option<[u8; 5]>,

        // size stored as a plain usize before the field if present
        #[option(plain, size(plain))]
        pub opt2: Option<&'a str>,

        // size deduced from the remaining buffer size
        #[size(deduced)]
        pub keyexpr: &'a str,
    }
}

#[derive(ZStruct)]
struct Field2<'a> {
    // Fields with fixed sized should not mention a size attribute. If nothing
    // is specified the size flavour is `none`. Which means that when decoding
    // the field should not rely on any size information.
    pub sn: u32,
    pub qos: u8,

    // Declare a 8-bit flag to store presence/size bits
    _flag: phantom::Flag<u8>,

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

    let b = (1usize << 5u8) - 1;

    let b: u32 = <u32 as Into<u32>>::into(a);
}
