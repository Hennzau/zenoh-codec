# zenoh-codec

A `#![no_std]`, `no_alloc` crate for defining structs, extensions, and messages for the Zenoh protocol — all in under 1.5 KLOC.

## Rules

The encoding rules of this codec are aligned with the existing `zenoh-codec` crate to ensure full compatibility with current implementations.

There are two kinds of structs you can declare:

* `#[derive(ZStruct)]`: a regular object that can be serialized/deserialized to and from a buffer.
* `#[derive(ZExt)]`: a specialized `ZStruct` that can be used as an extension within another `ZStruct`.

### Rules for `ZStruct`

* Only structs with no lifetime or a single lifetime parameter are supported.
* The following types implement `ZStruct`: `u8`, `u16`, `u32`, `u64`, `usize`, `[u8; N]`, `&str`, and `&[u8]`.
* `Option<T>` implements `ZStruct` if `T` does.
* Nested options are **not supported**.
* Each field can specify how to encode/decode its **size** and/or **presence** (for `Option<T>`) using attributes.
* Additionally, you can use the special **hstore** attribute on `T: Into<u8> + From<u8>` and `marker::Phantom` to indicate that the field should be encoded/decoded directly in the header field if present.

If no size attribute is specified, the default size flavor is `none`, meaning the field is encoded without any size information, and decoding must not rely on it.

#### Supported size flavors

* `none`: no size information is stored.
* `plain`: the size is stored as a plain `usize` before the field.
* `deduced`: the size is inferred from the remaining buffer size.
* `eflag = N`: the size is stored in `N` bits within a flag field (see below).
* `flag = N`: `(size - 1)` is stored in `N` bits within a flag field; when decoding, the size is incremented by 1. Better when the size cannot be zero.

#### Supported option flavors

* `plain`: presence is stored as a plain `u8` before the field.
* `flag`: presence is stored as one bit within a flag field.
* `header = MASK`: presence is stored in a header field using the provided bitmask.

### Supported hstore flavors

* `value = VALUE`: the field has no data in it but being present in the struct adds the specified `VALUE` (`u8`) to the header field during encoding.
* `mask = MASK, shift = SHIFT`: the field belongs to the struct but when encoding/decoding, its value is stored in the header field using the provided `MASK` (`u8`) and `SHIFT` (`u8`).

#### Flag and header fields

* A **flag field** is a `marker::Flag` (`u8`) declared before any field using it. Each flagged field consumes bits from left to right.
* A **header field** is a `marker::Header` (`u8`) declared at the start of the struct. Each field using it applies its bitmask to determine presence.

#### Phantom fields

* A **phantom field** is a `marker::Phantom` that must use `attributes` to specify its behavior. It does not occupy space in the struct.

#### Extension blocks

An extension block is declared using `marker::ExtBlockBegin` and `marker::ExtBlockEnd`.
Each extension inside must be an `Option<T>` where `T` implements `ZExt` (and `ZExtAttribute<ZStruct>`, see below).

The block’s presence must be encoded using one of the following option flavors, defined on the `ExtBlockBegin` field:

* `plain`: presence stored as a plain `u8` before the block.
* `flag`: presence stored as one bit in a flag field.
* `header = MASK`: presence stored in the header field using the provided bitmask.

**Note**: A block is present if at least one of its extensions is present. If no extensions are present, the block is considered absent.

### Rules for `ZExt`

`ZExt` follows the same rules as `ZStruct`.
When using a `ZExt` inside a `ZStruct`, you must declare its internal ID and whether it is **MANDATORY** or not, using the `zextattribute!` macro:

```rust
zextattribute!(impl<'a> ZExtType<'a>, ParentZStructType<'a>, <internal_id on 4 bits>, <mandatory bool>);
```

---

## Example

Declare your structs and extensions using the provided procedural macros:

```rust
#[derive(ZStruct, PartialEq, Debug)]
pub struct ZStruct1<'a> {
    // Fixed-size fields should not specify any size attribute.
    pub sn: u32,

    // Equivalent to not specifying any size attribute.
    #[size(none)]
    pub qos: u8,

    // Fixed-size byte array.
    pub array: [u8; 3],

    // Optional field with plain option flavor.
    #[option(plain)]
    pub opt: Option<[u8; 5]>,

    // Optional string with plain size and plain option flavor.
    #[option(plain, size(plain))]
    pub opt2: Option<&'a str>,

    // Size deduced from remaining buffer.
    #[size(deduced)]
    pub keyexpr: &'a str,
}

#[derive(ZStruct, PartialEq, Debug)]
struct ZStruct2<'a> {
    pub sn: u32,
    pub qos: u8,

    // Declare an 8-bit flag field.
    _flag: marker::Flag,

    // Optional field using bits in the flag field (1 for presence, 6 for size, keyexpr can be empty).
    #[option(flag, size(eflag = 6))]
    pub keyexpr: Option<&'a str>,

    #[size(plain)]
    pub field1: ZStruct1<'a>,

    #[option(flag, size(deduced))]
    pub field2: Option<ZStruct1<'a>>,
}

// A ZExt is a specialized ZStruct and follows the same rules.
// Depending on its fields, it becomes one of the ZExtKind variants:
// - Unit: no fields.
// - U64: only fixed-size numeric fields.
// - ZStruct: all other cases.
#[derive(ZExt, PartialEq, Debug)]
pub struct ZExt1<'a> {
    pub sn: u32,
    pub qos: u8,
    #[size(deduced)]
    pub keyexpr: &'a str,
}

#[derive(ZExt, PartialEq, Debug)]
pub struct ZExt2 {
    // Single fixed-size field → U64 specialization.
    pub sn: u32,
}

// A ZMsg is just a ZStruct. It can even contain extensions that are ZExts of another ZMsg.
#[derive(ZStruct, PartialEq, Debug)]
pub struct Msg1<'a> {
    _header: marker::Header,

    #[size(plain)]
    field: &'a str,

    // Presence of the extension block is stored in the header using the provided bitmask.
    #[option(header = 0b1000_0000)]
    _begin: marker::ExtBlockBegin,

    pub ext1: Option<ZExt1<'a>>,
    pub ext2: Option<ZExt2>,

    _end: marker::ExtBlockEnd,

    #[size(deduced)]
    payload: &'a [u8],
}

// Declare internal IDs and mandatory flags.
zextattribute!(impl<'a> ZExt1<'a>, Msg1<'a>, 0x1, true);
zextattribute!(impl<'a> ZExt2, Msg1<'a>, 0x2, true);
```

### Encoding and decoding

```rust
let x = Msg1 {
    _header: marker::Header,
    field: "hello",
    _begin: marker::ExtBlockBegin,
    ext1: Some(ZExt1 { sn: 42, qos: 1, keyexpr: "/foo/bar" }),
    ext2: Some(ZExt2 { sn: 7 }),
    _end: marker::ExtBlockEnd,
    payload: &[1, 2, 3, 4],
};

let mut data = [0u8; 128];
let mut writer = &mut data.as_mut_slice();

// Because Msg1 uses a deduced flavor, we must store its length for decoding.
let len = <Msg1 as ZStruct>::z_len(&x);
<Msg1 as ZStruct>::z_encode(&x, &mut writer).unwrap();

let mut reader = data.as_slice();
let decoded = <Msg1 as ZStruct>::z_decode(&mut reader.sub(len).unwrap()).unwrap();

assert_eq!(x, decoded);
```

---

## Maintainability

Effort has been made to keep the codebase as maintainable as possible, though writing clear `proc-macros` is inherently complex.
For clarity, each module (except the parsing module) should remain under **150 lines** to make the logic easy to follow.

---

## Error handling

Currently, the procedural macro **panics** when encountering invalid behavior.
This should ideally be replaced with proper error handling using `syn::Result`.
