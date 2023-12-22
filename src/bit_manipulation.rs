use std::ops::BitOrAssign;

#[inline]
pub fn build_bit_flag_set<I, T>(flag_translations: I) -> T
where
    I: IntoIterator<Item = (bool, T)>,
    T: Default + BitOrAssign,
{
    let mut bit_flag_set = T::default();

    for (needed, flag_bit_mask) in flag_translations.into_iter() {
        if needed {
            bit_flag_set |= flag_bit_mask;
        }
    }

    bit_flag_set
}

//TODO: Rather as generic functions with arithmetic trait bounds?
#[macro_export]
macro_rules! low_u8 {
    ($value:expr) => {
        ($value & 0xff) as u8
    };
}

#[macro_export]
macro_rules! high_u8 {
    ($value:expr) => {
        ($value >> 8 & 0xff) as u8
    };
}

#[macro_export]
macro_rules! low_u16 {
    ($value:expr) => {
        ($value & 0xffff) as u16
    };
}

#[macro_export]
macro_rules! high_u16 {
    ($value:expr) => {
        ($value >> 16 & 0xffff) as u16
    };
}

#[macro_export]
macro_rules! low_i16 {
    ($value:expr) => {
        low_u16!($value) as i16
    };
}

#[macro_export]
macro_rules! high_i16 {
    ($value:expr) => {
        high_u16!($value) as i16
    };
}

#[macro_export]
macro_rules! low_high_as_u16 {
    ($low:expr, $high:expr) => {
        ($low as u16 & 0xff) | (($high as u16 & 0xff) << 8)
    };
}

#[macro_export]
macro_rules! high_low_as_u16 {
    ($high:expr, $low:expr) => {
        low_high_as_u16!($low, $high)
    };
}

#[macro_export]
macro_rules! low_high_as_u32 {
    ($low:expr, $high:expr) => {
        ($low as u32 & 0xffff) | (($high as u32 & 0xffff) << 16)
    };
}

#[macro_export]
macro_rules! high_low_as_u32 {
    ($high:expr, $low:expr) => {
        low_high_as_u32!($low, $high)
    };
}
