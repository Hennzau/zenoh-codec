use zenoh_codec::{ZReaderExt, ZStruct, phantom};

use crate::inner::ZStruct1;

mod inner {
    use zenoh_codec::ZStruct;

    // Only no-lifetime or single-lifetime structs are supported.
    #[derive(ZStruct, PartialEq, Debug)]
    pub struct ZStruct1<'a> {
        // Fields with fixed sized should not mention a size attribute. If nothing
        // is specified the size flavour is `none`. Which means that when decoding
        // the field should not rely on any size information.
        pub sn: u32,

        // Just to illustrate that size(none) is equivalent to not specifying anything
        #[size(none)]
        pub qos: u8,

        // A fixed sized array of 3 bytes. You don't need to specify any size attribute here.
        pub array: [u8; 3],

        // An optional field should always specify its option flavour, otherwise it
        // will not compile. Nested options are not supported.
        //
        // There is no good error message for those two cases yet.
        #[option(plain)]
        pub opt: Option<[u8; 5]>,

        // size stored as a plain usize before the field if present
        #[option(plain, size(plain))]
        pub opt2: Option<&'a str>,

        // size deduced from the remaining buffer size
        #[size(deduced)]
        pub keyexpr: &'a str,
    }
}

#[derive(ZStruct, PartialEq, Debug)]
struct ZStruct2<'a> {
    // Fields with fixed sized should not mention a size attribute. If nothing
    // is specified the size flavour is `none`. Which means that when decoding
    // the field should not rely on any size information.
    pub sn: u32,
    pub qos: u8,

    // Declare a 8-bit flag to store presence/size bits. Available sizes are u8, u16, u32, u64. (Internally for > u8, it will encode it as VLE)
    _flag: phantom::Flag<u8>,

    // 7 bits to store the size in the flag
    #[option(flag, size(flag = 6))]
    pub keyexpr: Option<&'a str>,
    // size stored as a plain usize before the field
    #[size(plain)]
    pub field1: inner::ZStruct1<'a>,
    // optional presence stored as 1 bit in the flag and
    // 4 bits to store the size of the field in the flag as well
    #[option(flag, size(deduced))]
    pub field2: Option<inner::ZStruct1<'a>>,
}

fn main() {
    let struct1 = ZStruct1 {
        sn: 42,
        qos: 1,
        array: [1, 2, 3],
        opt: Some([4, 5, 6, 7, 8]),
        opt2: Some("hello"),
        keyexpr: "key==value",
    };

    let mut data = [0u8; 128];
    let mut writer = &mut data.as_mut_slice();

    let len = <ZStruct1 as ZStruct>::z_len(&struct1);
    <ZStruct1 as ZStruct>::z_encode(&struct1, &mut writer).unwrap();

    let mut reader = data.as_slice();
    let decoded_struct1 = <ZStruct1 as ZStruct>::z_decode(&mut reader.sub(len).unwrap()).unwrap();

    assert_eq!(struct1, decoded_struct1);

    let struct2 = ZStruct2 {
        sn: 43,
        qos: 0,

        _flag: phantom::Flag::new(),

        keyexpr: Some("another_key"),
        field1: struct1,
        field2: Some(ZStruct1 {
            sn: 44,
            qos: 2,
            array: [9, 8, 7],
            opt: None,
            opt2: None,
            keyexpr: "key3==value3",
        }),
    };

    let mut data = [0u8; 256];
    let mut writer = &mut data.as_mut_slice();

    let len = <ZStruct2 as ZStruct>::z_len(&struct2);
    <ZStruct2 as ZStruct>::z_encode(&struct2, &mut writer).unwrap();

    let mut reader = data.as_slice();
    let decoded_struct2 = <ZStruct2 as ZStruct>::z_decode(&mut reader.sub(len).unwrap()).unwrap();

    assert_eq!(struct2, decoded_struct2);
}
