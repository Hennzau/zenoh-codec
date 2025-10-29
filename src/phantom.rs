pub trait FlagType {}

impl FlagType for u8 {}
impl FlagType for u16 {}
impl FlagType for u32 {}
impl FlagType for u64 {}

pub struct Flag<T: FlagType> {
    _marker: core::marker::PhantomData<T>,
}

impl<T: FlagType> Flag<T> {
    pub fn new() -> Self {
        Self {
            _marker: core::marker::PhantomData,
        }
    }
}
