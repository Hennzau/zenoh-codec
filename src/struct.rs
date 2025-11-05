use crate::{ZReader, ZResult, ZWriter};

mod array;
mod bytes;
mod option;
mod str;
mod uint;

pub trait ZStructEncode {
    fn z_len(&self) -> usize;

    fn z_encode(&self, w: &mut ZWriter) -> ZResult<()>;
}

pub trait ZStructDecode<'a> {
    fn z_decode(r: &mut ZReader<'a>) -> ZResult<Self>
    where
        Self: Sized;
}
