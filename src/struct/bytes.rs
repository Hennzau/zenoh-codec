use crate::{ZReader, ZReaderExt, ZResult, ZWriter, ZWriterExt, r#struct::ZStruct};

impl ZStruct for &[u8] {
    fn z_len(&self) -> usize {
        self.len()
    }

    fn z_encode(&self, w: &mut ZWriter) -> ZResult<()> {
        w.write_exact(self)
    }

    type ZType<'a> = &'a [u8];

    fn z_decode<'a>(r: &mut ZReader<'a>) -> ZResult<Self::ZType<'a>> {
        r.read(r.remaining())
    }
}
