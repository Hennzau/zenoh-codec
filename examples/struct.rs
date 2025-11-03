use zenoh_codec::{ZReaderExt, ZStruct, marker};

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
        //
        // Option with presence stored as a plain u8 before the field
        #[option(plain)]
        pub opt: Option<[u8; 5]>,

        // size stored as a plain usize before the field if present
        // presence stored as a plain u8 before the size
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

    // Declare a 8-bit flag to store presence/size bits.
    _flag: marker::Flag,

    // optional presence stored as 1 bit in the flag
    // 6 bits to store the size in the flag
    #[option(flag, size(eflag = 6))]
    pub keyexpr: Option<&'a str>,

    // size stored as a plain usize before the field
    #[size(plain)]
    pub field1: inner::ZStruct1<'a>,

    // optional presence stored as 1 bit in the flag and
    // 4 bits to store the size of the field in the flag as well
    #[option(flag, size(deduced))]
    pub field2: Option<inner::ZStruct1<'a>>,
}

const FLAG_2: u8 = 0b0100_0000;

#[derive(ZStruct, PartialEq, Debug)]
struct ZStruct3<'a> {
    _header: marker::Header,

    // A header that will be used to store presence through bitmasking
    #[option(header = 0b1000_0000, size(plain))]
    pub keyexpr: Option<&'a str>,

    #[size(plain)]
    pub field1: inner::ZStruct1<'a>,

    #[option(header = FLAG_2, size(deduced))]
    pub field2: Option<ZStruct2<'a>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterestMode {
    Final,
    Current,
    Future,
    CurrentFuture,
}

impl From<u8> for InterestMode {
    fn from(value: u8) -> Self {
        match value {
            0 => InterestMode::Final,
            1 => InterestMode::Current,
            2 => InterestMode::Future,
            3 => InterestMode::CurrentFuture,
            _ => InterestMode::Final,
        }
    }
}

impl From<InterestMode> for u8 {
    fn from(value: InterestMode) -> Self {
        match value {
            InterestMode::Final => 0,
            InterestMode::Current => 1,
            InterestMode::Future => 2,
            InterestMode::CurrentFuture => 3,
        }
    }
}

#[derive(ZStruct, PartialEq, Debug)]
struct ZStruct4<'a> {
    _header: marker::Header,

    // A Phantom field that must use attribute to be encoded/decoded. Here we use the only one available for now: the
    // hstore(value/(mask & shift)) flavour. The `hstore(value)` flavour will always store the given const value into
    // the header when encoding, it will not try to read it so if you need to access it you will need to do it in
    // the upper layer or use the other `hstore(mask & shift)` flavour (see below).
    #[hstore(value = 0b1111_0000)]
    _id: marker::Phantom,

    // A u8 header storage using mask and shift to store/retrieve the value. The field type must implement From<u8> and Into<u8>
    // in order to be encoded/decoded properly.
    //
    // In this example we expect the InterestMode to be encoded into bits 2 and 3 of the header byte. But the Into<u8>
    // implementation will convert the InterestMode into values between 0 and 3 so we need to shift those bits by 2 to store them
    // into the correct position.
    #[hstore(mask = 0b0000_1100, shift = 2)]
    pub myhvalue: InterestMode,

    // Another u8 header storage example
    #[hstore(mask = 0b0000_0011, shift = 0)]
    pub myhvalue2: u8,

    #[size(deduced)]
    pub field1: inner::ZStruct1<'a>,
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

        _flag: marker::Flag,

        keyexpr: Some(""),
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

    let struct3 = ZStruct3 {
        _header: marker::Header,
        keyexpr: Some("key_expr"),
        field1: ZStruct1 {
            sn: 45,
            qos: 1,
            array: [10, 11, 12],
            opt: Some([13, 14, 15, 16, 17]),
            opt2: Some("world"),
            keyexpr: "key2==value2",
        },
        field2: Some(struct2),
    };

    let mut data = [0u8; 512];
    let mut writer = &mut data.as_mut_slice();

    let len = <ZStruct3 as ZStruct>::z_len(&struct3);
    <ZStruct3 as ZStruct>::z_encode(&struct3, &mut writer).unwrap();

    let mut reader = data.as_slice();
    let decoded_struct3 = <ZStruct3 as ZStruct>::z_decode(&mut reader.sub(len).unwrap()).unwrap();
    assert_eq!(struct3, decoded_struct3);

    let struct4 = ZStruct4 {
        _header: marker::Header,
        _id: marker::Phantom,
        myhvalue: InterestMode::Future,
        myhvalue2: 0b0000_0010,
        field1: ZStruct1 {
            sn: 46,
            qos: 2,
            array: [20, 21, 22],
            opt: None,
            opt2: Some("zenoh"),
            keyexpr: "key4==value4",
        },
    };

    let mut data = [0u8; 256];
    let mut writer = &mut data.as_mut_slice();

    let len = <ZStruct4 as ZStruct>::z_len(&struct4);
    <ZStruct4 as ZStruct>::z_encode(&struct4, &mut writer).unwrap();

    let mut reader = data.as_slice();
    let decoded_struct4 = <ZStruct4 as ZStruct>::z_decode(&mut reader.sub(len).unwrap()).unwrap();
    assert_eq!(struct4, decoded_struct4);

    let mut hreader = data.as_slice();
    let header = <u8 as ZStruct>::z_decode(&mut hreader).unwrap();
    assert_eq!(header & 0b1111_0000, 0b1111_0000);
}
