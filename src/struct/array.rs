use crate::{ZReader, ZReaderExt, ZResult, ZStructDecode, ZStructEncode, ZWriter, ZWriterExt};

impl<const N: usize> ZStructEncode for [u8; N] {
    fn z_len(&self) -> usize {
        N
    }

    fn z_encode(&self, w: &mut ZWriter) -> ZResult<()> {
        w.write_exact(self.as_slice())
    }
}

impl<'a, const N: usize> ZStructDecode<'a> for [u8; N] {
    fn z_decode(r: &mut ZReader<'a>) -> ZResult<Self> {
        let mut dst = [0u8; N];
        r.read_into(dst.as_mut_slice())?;
        Ok(dst)
    }
}
