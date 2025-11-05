#![no_std]

pub use zenoh_codec_derive::*;

#[cfg(test)]
mod tests;

pub mod r#struct;
pub use r#struct::*;

pub mod ext;
pub use ext::*;

pub type ZReader<'a> = &'a [u8];
pub type ZWriter<'a> = &'a mut [u8];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ZCodecError {
    CouldNotRead = 0,
    CouldNotWrite = 1,
    CouldNotParse = 2,

    FieldExceedsReservedSize = 3,

    MissingMandatoryExtension = 4,
}

pub type ZResult<T> = core::result::Result<T, ZCodecError>;

pub trait ZReaderExt<'a> {
    fn mark(&self) -> &'a [u8];
    fn rewind(&mut self, mark: &'a [u8]);

    fn remaining(&self) -> usize;
    fn can_read(&self) -> bool {
        self.remaining().gt(&0)
    }

    fn peek_u8(&self) -> ZResult<u8>;

    fn read(&mut self, len: usize) -> ZResult<&'a [u8]>;
    fn read_u8(&mut self) -> ZResult<u8>;

    fn read_into(&mut self, dst: &'_ mut [u8]) -> ZResult<usize>;

    fn sub(&mut self, len: usize) -> ZResult<ZReader<'a>> {
        let sub = self.read(len)?;
        Ok(sub)
    }
}

pub trait ZWriterExt<'a> {
    fn remaining(&self) -> usize;

    fn write(&mut self, src: &'_ [u8]) -> ZResult<usize>;
    fn write_u8(&mut self, value: u8) -> ZResult<()>;

    fn write_exact(&mut self, src: &'_ [u8]) -> ZResult<()>;
    fn write_slot(
        &mut self,
        len: usize,
        writer: impl FnOnce(&'_ mut [u8]) -> usize,
    ) -> ZResult<&'a [u8]>;
}

impl<'a> ZReaderExt<'a> for ZReader<'a> {
    fn mark(&self) -> &'a [u8] {
        self
    }

    fn rewind(&mut self, mark: &'a [u8]) {
        *self = mark;
    }

    fn remaining(&self) -> usize {
        self.len()
    }

    fn peek_u8(&self) -> ZResult<u8> {
        if !self.can_read() {
            return Err(ZCodecError::CouldNotRead);
        }

        Ok(unsafe { *self.get_unchecked(0) })
    }

    fn read_u8(&mut self) -> ZResult<u8> {
        if !self.can_read() {
            return Err(ZCodecError::CouldNotRead);
        }

        let value = unsafe { *self.get_unchecked(0) };
        *self = unsafe { self.get_unchecked(1..) };

        Ok(value)
    }

    fn read_into(&mut self, dst: &'_ mut [u8]) -> ZResult<usize> {
        let len = self.remaining().min(dst.len());
        if len == 0 {
            return Err(ZCodecError::CouldNotRead);
        }

        let (to_write, remain) = unsafe { self.split_at_unchecked(len) };
        unsafe {
            dst.get_unchecked_mut(..len).copy_from_slice(to_write);
        }

        *self = remain;

        Ok(len)
    }

    fn read(&mut self, len: usize) -> ZResult<&'a [u8]> {
        if self.len() < len {
            return Err(ZCodecError::CouldNotRead);
        }

        let (zbuf, remain) = unsafe { self.split_at_unchecked(len) };
        *self = remain;

        Ok(zbuf)
    }
}

impl<'a> ZWriterExt<'a> for ZWriter<'a> {
    fn remaining(&self) -> usize {
        self.len()
    }

    fn write_u8(&mut self, value: u8) -> ZResult<()> {
        if self.is_empty() {
            return Err(ZCodecError::CouldNotWrite);
        }

        unsafe {
            *self.get_unchecked_mut(0) = value;
            *self = core::mem::take(self).get_unchecked_mut(1..);
        }

        Ok(())
    }

    fn write(&mut self, src: &[u8]) -> ZResult<usize> {
        if src.is_empty() {
            return Ok(0);
        }
        let len = self.len().min(src.len());
        if len == 0 {
            return Err(ZCodecError::CouldNotWrite);
        }

        let (to_write, remain) = unsafe { core::mem::take(self).split_at_mut_unchecked(len) };
        to_write.copy_from_slice(unsafe { src.get_unchecked(..len) });
        *self = remain;

        Ok(len)
    }

    fn write_exact(&mut self, src: &[u8]) -> ZResult<()> {
        let len = self.write(src)?;

        if len < src.len() {
            return Err(ZCodecError::CouldNotWrite);
        }

        Ok(())
    }

    fn write_slot(
        &mut self,
        len: usize,
        writer: impl FnOnce(&mut [u8]) -> usize,
    ) -> ZResult<&'a [u8]> {
        if self.len() < len {
            return Err(ZCodecError::CouldNotWrite);
        }

        let written = writer(unsafe { self.get_unchecked_mut(..len) });

        if written > len {
            return Err(ZCodecError::CouldNotWrite);
        }

        let (slot, remain) = unsafe { core::mem::take(self).split_at_mut_unchecked(written) };
        *self = remain;

        Ok(slot)
    }
}
