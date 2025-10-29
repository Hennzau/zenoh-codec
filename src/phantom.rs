pub trait FlagType {}

impl FlagType for u8 {}
impl FlagType for u16 {}
impl FlagType for u32 {}
impl FlagType for u64 {}

#[derive(Debug, PartialEq)]
pub struct Flag<T: FlagType> {
    _marker: core::marker::PhantomData<T>,
}

impl<T: FlagType> Default for Flag<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: FlagType> Flag<T> {
    pub fn new() -> Self {
        Self {
            _marker: core::marker::PhantomData,
        }
    }
}
