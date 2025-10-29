use crate::{ZReader, ZResult, ZWriter};

mod array;
mod bytes;
mod option;
mod str;
mod uint;

/// A trait representing a field that can be encoded and decoded in the Zenoh protocol.
pub trait ZStruct {
    /// Returns the length in bytes of the encoded field.
    fn z_len(&self) -> usize;

    /// Encodes the field into the provided writer. It ensures that exactly `z_len()` bytes are written.
    fn z_encode(&self, w: &mut ZWriter) -> ZResult<()>;

    /// The type produced by decoding the field.
    type ZType<'a>: Sized;

    /// Decodes the field from the provided reader without requiring a sized reader.
    fn z_decode<'a>(r: &mut ZReader<'a>) -> ZResult<Self::ZType<'a>>;
}
