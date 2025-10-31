# zenoh-codec

A `#![no_std]`, `no_alloc` crate to write structs, extensions and messages for the Zenoh protocol in less than 1.5kloc.

## Rules

The rules of this codec are based on the current `zenoh-codec` crate to ensure compatibility with existing implementations.

There are 2 kind of structs you can declare:
- `#[derive(ZStruct)]`: this is a regular object that you can serialize/deserialize in a buffer.
- `#[derive(ZExt)]`: this is a specialized `ZStruct` that can be used as an extension in a `ZStruct` later.

The rules for a `ZStruct` are simple:
- Only no-lifetime or single-lifetime structs are supported.
- u8, u16, u32, u64, usize, [u8; N], &str and &[u8] implement `ZStruct`.
- `Option<T>` implements `ZStruct` if T implements `ZStruct`.
- Nested options are not supported.
- Each field can specify how to encode/decode its size and/or presence (for `Option<T>`) using attributes:
  - If no size attribute is specified the size flavour is `none`. Which means that when decoding the field should not rely on any size information.

  - Supported size flavours are:
    - `none`: no size information is stored.
    - `plain`: size is stored as a plain `usize` before the field.
    - `deduced`: size is deduced from the remaining buffer size.
    - `eflag = N`: size is stored in N bits inside a flag field (see below).
    - `flag = N`: (size - 1) is stored in N bits inside a flag field (see below), when decoding the final size is incremented by 1. It is useful when the size cannot be zero.

  - Supported option flavours are:
    - `plain`: presence is stored as a plain `u8` before the field.
    - `flag`: presence is stored as 1 bit inside a flag field (see below).
    - `header = MASK`: presence is stored in the header field using the provided bitmask.

- A flag field is a field of type `marker::Flag` (u8) declared in the struct before any field using it. Each field using the flag attribute will consume bits from the flag from left to right.

- A header field is a field of type `marker::Header` (u8) declared at the beginning of the struct. Each field using the header attribute will apply its bitmask to the header to determine presence.

- An extension block is declared using a field of type `marker::ExtBlockBegin` and `marker::ExtBlockEnd`. Each extension inside the block should be an `Option<T>` where T implements `ZExt`.

An extension block must specify how to encode/decode the presence/non presence of at least one extension inside using the option attribute on the `ExtBlockBegin` field. Supported flavours are:
  - `plain`: presence is stored as a plain `u8` before the block.
  - `flag`: presence is stored as 1 bit inside a flag field (see above).
  - `header = MASK`: presence is stored in the header field using the provided bitmask.

The rules for a `ZExt` are the same as for a `ZStruct`. However when you will want to use a `ZExt` inside
a `ZStruct` you will need to declare its internal ID and if it is MANDATORY or not using the provided `zextattribute!` macro:

```Rust
zextattribute!(impl<'a> ZExtType<'a>, ParentZStructType<'a>, <internal_id on 4 bits>, <mandatory bool>);
```

## Example

Declare your structs and extensions using the provided `proc-macros`.

```Rust
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
    pub field1: ZStruct1<'a>,

    // optional presence stored as 1 bit in the flag and
    // 4 bits to store the size of the field in the flag as well
    #[option(flag, size(deduced))]
    pub field2: Option<ZStruct1<'a>>,
}

// A ZExt is a specialized ZStruct so it must respect all the rules defined for ZStructs.
// Depending on the fields present in the struct the ZExt will be specialized to one of the
// three kinds defined in ZExtKind.
//
// - Unit: if the struct has no fields.
// - U64: if the struct has only fixed size fields (u16, u32, u64, usize).
// - ZStruct: the rest of the cases.
#[derive(ZExt, PartialEq, Debug)]
pub struct ZExt1<'a> {
    pub sn: u32,
    pub qos: u8,

    #[size(deduced)]
    pub keyexpr: &'a str,
}

#[derive(ZExt, PartialEq, Debug)]
pub struct ZExt2 {
    // Only one fixed size field to be specialized as U64 kind.
    pub sn: u32,
}

// A ZMsg is in fact a regular ZStruct. So yes you can have a ZMsg which is in fact a ZExt of another ZMsg.
#[derive(ZStruct, PartialEq, Debug)]
pub struct Msg1<'a> {
    // A header acts like a flag but instead of fulling it from the left to the right, each field can apply a bitmask
    _header: marker::Header,

    #[size(plain)]
    field: &'a str,

    // Declare an extension block. Precise how to encode the presence/non presence
    // of at least one extension inside. (available are flag, header or plain)
    #[option(header = 0b1000_0000)]
    _begin: marker::ExtBlockBegin,

    // Extensions in an ExtBlock should always be an option. Failing to do so will result in
    // a compile error but there is no good error message yet.
    pub ext1: Option<ZExt1<'a>>,
    pub ext2: Option<ZExt2>,

    // You should always mark the end of an ext block.
    _end: marker::ExtBlockEnd,

    // You can have other fields after the ext block. You can even have multiple ext blocks.
    #[size(deduced)]
    payload: &'a [u8],
}

// For this ZStruct/ZMsg declare the internal ID and MANDATORY flag.
zextattribute!(impl<'a> ZExt1<'a>, Msg1<'a>, 0x1, true);
zextattribute!(impl<'a> ZExt2, Msg1<'a>, 0x2, true);
```

Once declared you can use the generated methods to encode/decode your structs and extensions.

```Rust
let x = Msg1 {
    _header: marker::Header,
    field: "hello",
    _begin: marker::ExtBlockBegin,
    ext1: Some(ZExt1 {
        sn: 42,
        qos: 1,
        keyexpr: "/foo/bar",
    }),
    ext2: Some(ZExt2 { sn: 7 }),
    _end: marker::ExtBlockEnd,
    payload: &[1, 2, 3, 4],
};

let mut data = [0u8; 128];
let mut writer = &mut data.as_mut_slice();

// Because Msg1 uses a deduced flavor for its last field we need to store the length to decode it later.
let len = <Msg1 as ZStruct>::z_len(&x);
<Msg1 as ZStruct>::z_encode(&x, &mut writer).unwrap();

let mut reader = data.as_slice();
// Resize the reader to only the encoded part so that deduced sizes work correctly.
let decoded = <Msg1 as ZStruct>::z_decode(&mut reader.sub(len).unwrap()).unwrap();

assert_eq!(x, decoded);
```

## Maintainability

I tried my best to keep the code as maintainable as possible but it's not easy to write easy to follow
`proc-macros`.

For simplicity, each file (but the parsing module) should be less than 150 lines of code so that each part of the process can be
easily understood.

## Error handling

Currently, the `proc-macro` panics when a wrong behavior is detected. This is not ideal we should use `syn::Result` instead.
