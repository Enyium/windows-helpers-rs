#![cfg(feature = "f_Win32_Foundation")]

use crate::windows;
use windows::Win32::Foundation::{E_FAIL, LPARAM};

pub trait BoolExt {
    /// Like `BOOL::ok()`, but returning an `Error` with `HRESULT` `E_FAIL` instead of calling `GetLastError()`.
    fn ok_or_e_fail(self) -> windows::core::Result<()>;
}

impl BoolExt for windows::Win32::Foundation::BOOL {
    fn ok_or_e_fail(self) -> windows::core::Result<()> {
        if self.as_bool() {
            Ok(())
        } else {
            Err(E_FAIL.into())
        }
    }
}

pub trait LParamExt {
    unsafe fn cast_to_ref<T>(&self) -> &T;
    unsafe fn cast_to_mut<T>(&mut self) -> &mut T;
}

impl LParamExt for LPARAM {
    unsafe fn cast_to_ref<T>(&self) -> &T {
        &*(self.0 as *const T)
    }

    unsafe fn cast_to_mut<T>(&mut self) -> &mut T {
        &mut *(self.0 as *mut T)
    }
}
