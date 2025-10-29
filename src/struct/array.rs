use crate::{ZReader, ZReaderExt, ZResult, ZWriter, ZWriterExt, r#struct::ZStruct};

impl<const N: usize> ZStruct for [u8; N] {
    fn z_len(&self) -> usize {
        N
    }

    fn z_encode(&self, w: &mut ZWriter) -> ZResult<()> {
        w.write_exact(self.as_slice())
    }

    type ZType<'a> = [u8; N];

    fn z_decode<'a>(r: &mut ZReader<'a>) -> ZResult<Self::ZType<'a>> {
        let mut dst = [0u8; N];
        r.read_into(dst.as_mut_slice())?;
        Ok(dst)
    }
}
