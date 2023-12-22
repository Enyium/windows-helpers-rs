#![cfg(feature = "f_Win32_UI_WindowsAndMessaging")]

use crate::windows;
use std::mem;
use windows::Win32::{Foundation::LPARAM, UI::WindowsAndMessaging::TIMERPROC};

pub trait TimerProcExt {
    unsafe fn from_lparam(lparam: LPARAM) -> Self;
}

impl TimerProcExt for TIMERPROC {
    /// It's yet to be confirmed that the transmute works. Create an issue if it works or doesn't.
    unsafe fn from_lparam(lparam: LPARAM) -> Self {
        if lparam.0 != 0 {
            Some(mem::transmute(lparam.0))
        } else {
            None
        }
    }
}
