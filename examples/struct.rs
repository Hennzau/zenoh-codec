use zenoh_codec::{ZReaderExt, ZStruct, ZStructDecode, ZStructEncode};

// Only no-lifetime or single-lifetime structs are supported.
//
// **Note**: this struct uses the `size = remain` flavour. This means that when decoding this struct, the
// reader should be bounded to the exact size of the struct, otherwise decoding will fail. This is automatically done
// if the upper layer stores the size of this struct somewhere (e.g., in a prefixed size or in a header) or
// if it uses the remain flavour itself which will expect the upper upper layer to have bounded the reader).
#[derive(ZStruct, PartialEq, Debug)]
pub struct ZStruct1<'a> {
    // Numeric fields should not specify any attribute.
    pub sn: u32,
    pub qos: u8,

    // A fixed sized array of 3 bytes. You don't need to specify any attribute.
    pub array: [u8; 3],

    // An optional field must specify its presence flavour. Nested options are not supported.
    //
    // Option with presence stored as a prefix (u8) before the field
    #[zenoh(presence = prefixed)]
    pub opt: Option<[u8; 5]>,

    // Option with presence stored as a prefix (u8) before the field
    // If present, the size is also stored as a prefix (usize) before the field (and after the presence byte)
    #[zenoh(presence = prefixed, size = prefixed)]
    pub opt2: Option<&'a str>,

    // A string field with no stored size. The field will consume all the remaining bytes when decoding.
    #[zenoh(size = remain)]
    pub keyexpr: &'a str,
}

// A header declared with 3 slots, two single bits and one 6-bits.
#[derive(ZStruct, PartialEq, Debug)]
#[zenoh(header = "A|B|S:6")]
struct ZStruct2<'a> {
    pub sn: u32,

    // Presence stored using the bitmasking of header 'A'.
    // If present the size is stored using the 6 bits of header 'S'.
    //
    // The field can be empty, so when encoding the size in the header it will not subtract 1 from the size.
    #[zenoh(presence = header(A), size = header(S), maybe_empty)]
    pub keyexpr: Option<&'a str>,

    // Size stored as a prefix (usize) before the field
    #[zenoh(size = prefixed)]
    pub field1: ZStruct1<'a>,

    // Presence stored using the bitmasking of header 'B'.
    // Size will be deduced from the remaining bytes.
    #[zenoh(presence = header(B), size = remain)]
    pub field2: Option<ZStruct1<'a>>,
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

// A struct with a header with assigned fixed bits and two u8 fields
//  stored in the header.
#[derive(ZStruct, PartialEq, Debug)]
#[zenoh(header = "ID:4=0xA|I:2|U:2")]
struct ZStruct3<'a> {
    // Will store the value of this field directly in the header (slot `I`).
    #[zenoh(header = I)]
    pub myhvalue: InterestMode,

    // Will store the value of this field directly in the header (slot `U`).
    #[zenoh(header = U)]
    pub myhvalue2: u8,

    #[zenoh(size = remain)]
    pub field1: ZStruct1<'a>,
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

    let len = <_ as ZStructEncode>::z_len(&struct1);
    <_ as ZStructEncode>::z_encode(&struct1, &mut writer).unwrap();

    let mut reader = data.as_slice();
    let decoded_struct1 = <_ as ZStructDecode>::z_decode(&mut reader.sub(len).unwrap()).unwrap();

    assert_eq!(struct1, decoded_struct1);

    let struct2 = ZStruct2 {
        sn: 43,

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

    let len = <_ as ZStructEncode>::z_len(&struct2);
    <_ as ZStructEncode>::z_encode(&struct2, &mut writer).unwrap();

    let mut reader = data.as_slice();
    let decoded_struct2 = <_ as ZStructDecode>::z_decode(&mut reader.sub(len).unwrap()).unwrap();

    assert_eq!(struct2, decoded_struct2);

    let struct3 = ZStruct3 {
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

    let len = <_ as ZStructEncode>::z_len(&struct3);
    <_ as ZStructEncode>::z_encode(&struct3, &mut writer).unwrap();

    let mut reader = data.as_slice();
    let header = reader.peek_u8().unwrap();
    assert_eq!((header & 0b1111_0000) >> 4, ZStruct3::ID);

    let decoded_struct3 = <_ as ZStructDecode>::z_decode(&mut reader.sub(len).unwrap()).unwrap();
    assert_eq!(struct3, decoded_struct3);
}
