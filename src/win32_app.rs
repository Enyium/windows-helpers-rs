//! Helpers to simplify some tedious aspects of a Win32 application, while still staying low-level or maintaining a low-level connection.
//!
//! Activate feature `windows_<version>_win32_app` (available from `windows` v0.52 onwards).

pub mod msg_loop;
pub mod window;
