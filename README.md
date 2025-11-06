# zenoh-codec

A `#![no_std]`, `no_alloc` crate for defining structs, extensions, and messages for the Zenoh protocol — all in under 2.0 KLOC.

## Rules

The encoding rules of this codec are aligned with the existing `zenoh-codec` crate to ensure full compatibility with current implementations.

There are two kinds of structs you can declare:

* `#[derive(ZStruct)]`: a regular object that can be serialized/deserialized to and from a buffer.
* `#[derive(ZExt)]`: a specialized `ZStruct` that can be used as an extension within another `ZStruct`.

### Rules for `ZStruct`

* Only structs with no lifetime or a single lifetime parameter are supported.
* The following types implement `ZStruct`: `u8`, `u16`, `u32`, `u64`, `usize`, `[u8; N]`, `&str`, and `&[u8]`.
* All types that implement `ZStruct` can be used as fields within a `ZStruct`.
* Fields can also be `Option<T>` where `T: ZStruct`. **Note**: `Option<T>` doest not implement `ZStruct` itself.
* Nested options are **not supported**.
* Additionnaly you may need to specify attributes for certain fields using the `#[zenoh()]` attribute (see below).

### `ZStruct` attribute

The `#[zenoh(...)]` attribute can be used above the struct declaration to specify the following options:

* `header = "_|_|_|_|_|_|_|_"`: declares a 8bit header field for the struct. You can specify which bits are used by replacing `_` with an identifier. You can specify sizes and even affect values. For example:
  * `header = "Z|S:7|"`: declares a header where bit 0 is named `Z` and bits 1 to 7 are named `S`. This will generate constants `Z: u8 = 0b1000_000` and `S: u8 = 0b0111_1111`.
  * `header = "Z=1|S:7|"`: declares a header where bit 0 is named `Z` and is always set to `1`, and bits 1 to 7 are named `S`.
  * `header = "_:8"`: declares a header with no named bits.

  A header is required if any field uses `header(MASK)` size or presence flavours, or if the struct contains an extension block. In this last case the header must start with a `Z` bit.

### Field attributes

Field attributes are specified using the `#[zenoh(...)]` attribute above the field declaration.

* `size = <...>`: specifies how the size of the field is encoded/decoded. Possible values:
  * `prefixed`: size is stored as a plain `usize` before the field.
  * `remain`: size is deduced from the remaining reader length.
  * `header(MASK)`: size is stored in the header field using the provided slot in the header. **Note**: it will assume the value cannot be empty. If the value can be empty you should add the `maybe_empty` attribute as well.

* `presence = <...>`: specifies how the presence of the field is encoded/decoded for `Option<T>` fields. Possible values:
  * `prefuxed`: presence stored as a plain `u8` before the field.
  * `header(MASK)`: presence stored in the header field using the provided bitmask.

* `maybe_empty`: indicates that the field can be empty (size 0). This is only ysed when using `header(MASK)` size flavour.
* `ext = <ID>`: indicates that the field is an extension with the given internal ID.
* `mandatory`: indicates that the extension is mandatory. This is only used for extensions.
* `default = <...>`: specifies a default value for the field when the field is an extension. It will not encode it if the value matches the default and when decoding it will set the field to the default if the extension is absent.

**Note**: `#[zenoh(ext = <ID>)]` fields must be grouped together in the struct.

### Rules for `ZExt`

`ZExt` follows the same rules as `ZStruct`.

---

## Example

Declare your structs and extensions using the provided procedural macros:

```rust
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

// A ZExt is a specialized ZStruct that can be used inside another ZStruct as an extension block.
// The kind of ZExt is determined by its fields:
// - If it has one fixed size field, it is specialized into U64.
// - If it has no fields, it is specialized into Unit.
// - Otherwise, it is specialized into ZStruct.
//
// `ZExt1` is a regular ZStruct, so it will be specialized into ZStruct.
#[derive(ZExt, PartialEq, Debug)]
pub struct ZExt1<'a> {
    pub sn: u32,
    pub qos: u8,

    #[zenoh(size = remain)]
    pub keyexpr: &'a str,
}

#[derive(ZExt, PartialEq, Debug)]
pub struct ZExt2 {
    // One fixed size field to be specialized into U64.
    pub sn: u32,
}

const DEFAULT_ZEXT2: ZExt2 = ZExt2 { sn: 0 };

// Using `ZExt`'s inside a `ZStruct` requires to use a header with a `Z` bit on the left side of the header.
#[derive(ZStruct, PartialEq, Debug)]
#[zenoh(header = "Z|_:7")]
pub struct Msg1<'a> {
    #[zenoh(size = prefixed)]
    field: &'a str,

    // Each ext field must precise its ext id.
    #[zenoh(ext = 0x1)]
    pub ext1: Option<ZExt1<'a>>,
    // If you don't want to use Option, you can use ZExt directly but you must provide a default value.
    #[zenoh(ext = 0x2, default = DEFAULT_ZEXT2)]
    pub ext2: ZExt2,

    // You can have other fields after the ext block.
    #[zenoh(size = remain)]
    payload: &'a [u8],
}
```

### Encoding and decoding

```rust
let x = Msg1 {
    field: "hello",
    ext1: Some(ZExt1 { sn: 42, qos: 1, keyexpr: "/foo/bar" }),
    ext2: ZExt2 { sn: 7 },
    payload: &[1, 2, 3, 4],
};

let mut data = [0u8; 128];
let mut writer = &mut data.as_mut_slice();

// Because Msg1 uses a deduced flavour, we must store its length for decoding.
let len = <_ as ZStructEncode>::z_len(&x);
<_ as ZStructEncode>::z_encode(&x, &mut writer).unwrap();

let mut reader = data.as_slice();
let decoded = <_ as ZStructDecode>::z_decode(&mut reader.sub(len).unwrap()).unwrap();

assert_eq!(x, decoded);
```

---

## Maintainability

Effort has been made to keep the codebase as maintainable as possible, though writing clear `proc-macros` is inherently complex.
For clarity, each module (except the parsing module) should remain under **200 lines** to make the logic easy to follow.
