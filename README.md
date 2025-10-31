# zenoh-codec

A `#![no_std]`, `no_alloc` crate to write structs, extensions and messages for the Zenoh protocol.

## Example



## Maintainability

I tried my best to keep the code as maintainable as possible but it's not easy to write easy to follow
`proc-macros`.

For simplicity, each file (but the parsing module) should be less than 150 lines of code so that each part of the process can be
easily understood.

## Error handling

Currently, the `proc-macro` panics when a wrong behavior is detected. This is not ideal we should use `syn::Result` instead.
