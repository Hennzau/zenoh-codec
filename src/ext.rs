use crate::{ZCodecError, ZReader, ZReaderExt, ZResult, ZStruct, ZWriter};

const KIND_MASK: u8 = 0b0110_0000;
const ID_MASK: u8 = 0b0000_1111;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ZExtKind {
    Unit = 0b00 << 5,
    U64 = 0b01 << 5,
    ZStruct = 0b10 << 5,
}

impl From<ZExtKind> for u8 {
    fn from(kind: ZExtKind) -> Self {
        kind as u8
    }
}

impl TryFrom<u8> for ZExtKind {
    type Error = ZCodecError;

    fn try_from(value: u8) -> ZResult<Self> {
        match value & KIND_MASK {
            0b0000_0000 => Ok(ZExtKind::Unit),
            0b0010_0000 => Ok(ZExtKind::U64),
            0b0100_0000 => Ok(ZExtKind::ZStruct),
            _ => Err(ZCodecError::CouldNotParse),
        }
    }
}

pub trait ZExt: ZStruct {
    const KIND: ZExtKind;

    fn z_len(&self) -> usize {
        match Self::KIND {
            ZExtKind::Unit | ZExtKind::U64 => <Self as ZStruct>::z_len(self),
            ZExtKind::ZStruct => {
                <usize as ZStruct>::z_len(&<Self as ZStruct>::z_len(self))
                    + <Self as ZStruct>::z_len(self)
            }
        }
    }

    fn z_encode(&self, w: &mut ZWriter) -> ZResult<()> {
        if Self::KIND == ZExtKind::ZStruct {
            <usize as ZStruct>::z_encode(&<Self as ZStruct>::z_len(self), w)?;
        }

        <Self as ZStruct>::z_encode(self, w)
    }

    fn z_decode<'a>(r: &mut ZReader<'a>) -> ZResult<Self::ZType<'a>> {
        if Self::KIND == ZExtKind::ZStruct {
            let len = <usize as ZStruct>::z_decode(r)?;
            <Self as ZStruct>::z_decode(&mut <ZReader as ZReaderExt>::sub(r, len)?)
        } else {
            <Self as ZStruct>::z_decode(r)
        }
    }
}

const FLAG_MANDATORY: u8 = 1 << 4;
const FLAG_MORE: u8 = 1 << 7;

/// Declare an extension as an attribute for <T>.
pub trait ZExtAttribute<T>: ZExt + ZStruct {
    const ID: u8;
    const MANDATORY: bool;

    const HEADER: u8 =
        (Self::ID | Self::KIND as u8) | if Self::MANDATORY { FLAG_MANDATORY } else { 0 };

    fn z_len(&self) -> usize {
        1 + <Self as ZExt>::z_len(self)
    }

    fn z_encode(&self, w: &mut ZWriter, more: bool) -> ZResult<()> {
        let header = Self::HEADER | if more { FLAG_MORE } else { 0 };

        <u8 as ZStruct>::z_encode(&header, w)?;
        <Self as ZExt>::z_encode(self, w)
    }

    fn z_decode<'a>(r: &mut ZReader<'a>) -> ZResult<Self::ZType<'a>> {
        let _ = <u8 as ZStruct>::z_decode(r)?;

        Ok(<Self as ZExt>::z_decode(r)?)
    }
}

pub fn skip_ext(r: &mut ZReader, kind: ZExtKind) -> ZResult<()> {
    let _ = <u8 as ZStruct>::z_decode(r)?;

    match kind {
        ZExtKind::Unit => {}
        ZExtKind::U64 => {
            let _ = <u64 as ZStruct>::z_decode(r)?;
        }
        ZExtKind::ZStruct => {
            let len = <usize as ZStruct>::z_decode(r)?;
            let _ = <ZReader as ZReaderExt>::sub(r, len)?;
        }
    }

    Ok(())
}

pub fn decode_ext_header(r: &mut ZReader) -> ZResult<(u8, ZExtKind, bool, bool)> {
    let header = r.get_u8()?;

    let id = header & ID_MASK;
    let kind = ZExtKind::try_from(header & KIND_MASK)?;
    let mandatory = (header & FLAG_MANDATORY) != 0;
    let more = (header & FLAG_MORE) != 0;

    Ok((id, kind, mandatory, more))
}

#[macro_export]
macro_rules! zextattribute {
    (impl<'a> $ext:ty, $t:ty, $id:expr, $m:expr) => {
        impl<'a> zenoh_codec::ZExtAttribute<$t> for $ext {
            const ID: u8 = $id;
            const MANDATORY: bool = $m;
        }
    };

    ($ext:ty, $t:ty, $id:expr, $m:expr) => {
        impl zenoh_codec::ZExtAttribute<$t> for $ext {
            const ID: u8 = $id;
            const MANDATORY: bool = $m;
        }
    };
}
