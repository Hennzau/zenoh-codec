use crate::{ZReader, ZReaderExt, ZResult, ZStruct, ZWriter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ZExtKind {
    Unit = 0b00 << 5,
    U64 = 0b01 << 5,
    ZStruct = 0b10 << 5,
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
            <usize as ZStruct>::z_encode(&<Self as ZExt>::z_len(self), w)?;
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

pub trait ZExtMsg<Msg>: ZExt + ZStruct {
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

    fn z_decode<'a>(r: &mut ZReader<'a>) -> ZResult<(Self::ZType<'a>, bool)> {
        let header = <u8 as ZStruct>::z_decode(r)?;
        let more = (header & FLAG_MORE) != 0;

        Ok((<Self as ZExt>::z_decode(r)?, more))
    }
}
