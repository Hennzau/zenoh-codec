use crate::{ZReader, ZResult, ZWriter, field::ZField};

impl<T: ZField> ZField for Option<T> {
    fn z_len(&self) -> usize {
        match self {
            Some(value) => value.z_len(),
            None => 0,
        }
    }

    fn z_encode(&self, w: &mut ZWriter) -> ZResult<()> {
        if let Some(value) = self {
            value.z_encode(w)?;
        }
        Ok(())
    }

    type ZType<'a> = T::ZType<'a>;

    fn z_decode<'a>(r: &mut ZReader<'a>) -> ZResult<Self::ZType<'a>> {
        T::z_decode(r)
    }
}
