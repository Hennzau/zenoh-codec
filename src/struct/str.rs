use crate::{ZCodecError, ZReader, ZReaderExt, ZResult, ZWriter, ZWriterExt, r#struct::ZStruct};

impl ZStruct for &'_ str {
    fn z_len(&self) -> usize {
        self.as_bytes().len()
    }

    fn z_encode(&self, w: &mut ZWriter) -> ZResult<()> {
        w.write_exact(self.as_bytes())
    }

    type ZType<'a> = &'a str;

    fn z_decode<'a>(r: &mut ZReader<'a>) -> ZResult<Self::ZType<'a>> {
        let bytes = r.read(r.remaining())?;

        core::str::from_utf8(bytes).map_err(|_| ZCodecError::CouldNotParse)
    }
}
