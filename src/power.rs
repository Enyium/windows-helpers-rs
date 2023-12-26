#![cfg(feature = "f_Win32_System_Power")]

use crate::{foundation::BoolExt, windows};
use std::{mem, ptr};
use windows::Win32::{Foundation::BOOL, System::Power::POWERBROADCAST_SETTING};

pub trait PowerBroadcastSettingExt {
    unsafe fn cast_data<T>(&self) -> windows::core::Result<&T>;
}

impl PowerBroadcastSettingExt for POWERBROADCAST_SETTING {
    unsafe fn cast_data<T>(&self) -> windows::core::Result<&T> {
        BOOL::from(self.DataLength == mem::size_of::<T>() as u32).ok_or_e_fail()?;
        Ok(&*ptr::addr_of!(self.Data).cast::<T>())
    }
}
