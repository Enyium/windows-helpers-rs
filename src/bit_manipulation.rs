use crate::windows;
use std::{mem, ops::BitOrAssign};
use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};

#[inline]
pub fn build_bit_flag_set<I, T>(flag_translations: I) -> T
where
    I: IntoIterator<Item = (bool, T)>,
    T: Default + BitOrAssign,
{
    //! Translates booleans to bits in a bit set.

    let mut bit_flag_set = T::default();

    for (needed, flag_bit_mask) in flag_translations.into_iter() {
        if needed {
            bit_flag_set |= flag_bit_mask;
        }
    }

    bit_flag_set
}

/// A trait concerned with the least significant 16 bits of integer types.
pub trait Width16BitPortion
where
    Self: Sized,
{
    fn from_low_high_u8(low: u8, high: u8) -> Self;

    #[inline]
    fn from_high_low_u8(high: u8, low: u8) -> Self {
        Self::from_low_high_u8(low, high)
    }

    fn low_u8(self) -> u8;
    fn high_u8(self) -> u8;
}

macro_rules! impl_width_16_portion {
    ($($type:ty),*) => {
        $(
            impl Width16BitPortion for $type {
                #[inline]
                fn from_low_high_u8(low: u8, high: u8) -> Self {
                    ((low as u16 & 0xff) | ((high as u16 & 0xff) << 8)) as Self
                }

                #[inline]
                fn low_u8(self) -> u8 {
                    (self & 0xff) as u8
                }

                #[inline]
                fn high_u8(self) -> u8 {
                    (self >> 8 & 0xff) as u8
                }
            }
        )*
    };
}

impl_width_16_portion!(u16, i16);

/// A trait concerned with the least significant 32 bits of integer types.
pub trait Width32BitPortion
where
    Self: Sized,
{
    fn from_low_high_u16(low: u16, high: u16) -> Self;

    #[inline]
    fn from_high_low_u16(high: u16, low: u16) -> Self {
        Self::from_low_high_u16(low, high)
    }

    #[inline]
    fn from_low_high_i16(low: i16, high: i16) -> Self {
        Self::from_low_high_u16(low as u16, high as u16)
    }

    #[inline]
    fn from_high_low_i16(high: i16, low: i16) -> Self {
        Self::from_low_high_i16(low, high)
    }

    fn low_u16(self) -> u16;
    fn high_u16(self) -> u16;

    #[inline]
    fn low_i16(self) -> i16 {
        self.low_u16() as i16
    }

    #[inline]
    fn high_i16(self) -> i16 {
        self.high_u16() as i16
    }
}

macro_rules! impl_width_32_portion {
    ($($type:ty),*) => {
        $(
            impl Width32BitPortion for $type {
                #[inline]
                fn from_low_high_u16(low: u16, high: u16) -> Self {
                    ((low as u32 & 0xffff) | ((high as u32 & 0xffff) << 16)) as Self
                }

                #[inline]
                fn low_u16(self) -> u16 {
                    (self & 0xffff) as u16
                }

                #[inline]
                fn high_u16(self) -> u16 {
                    (self >> 16 & 0xffff) as u16
                }
            }
        )*
    };
}

impl_width_32_portion!(u32, i32, usize, isize);

macro_rules! impl_width_32_portion_for_ptr_sized_1_tuple {
    ($($type:ty),*) => {
        $(
            impl Width32BitPortion for $type {
                #[inline]
                fn from_low_high_u16(low: u16, high: u16) -> Self {
                    unsafe { mem::transmute(usize::from_low_high_u16(low, high)) }
                }

                #[inline]
                fn low_u16(self) -> u16 {
                    self.0.low_u16()
                }

                #[inline]
                fn high_u16(self) -> u16 {
                    self.0.high_u16()
                }
            }
        )*
    };
}

#[cfg(feature = "f_Win32_Foundation")]
impl_width_32_portion_for_ptr_sized_1_tuple!(WPARAM, LPARAM, LRESULT);
