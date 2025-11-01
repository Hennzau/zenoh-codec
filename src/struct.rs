use crate::{ZReader, ZResult, ZWriter};

mod array;
mod bytes;
mod option;
mod str;
mod uint;

pub trait ZStruct {
    fn z_len(&self) -> usize;

    fn z_encode(&self, w: &mut ZWriter) -> ZResult<()>;

    type ZType<'a>: Sized;

    fn z_decode<'a>(r: &mut ZReader<'a>) -> ZResult<Self::ZType<'a>>;
}
