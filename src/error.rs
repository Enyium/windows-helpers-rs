use crate::windows::{self, core::HRESULT};
use windows::Win32::Foundation::E_FAIL;

//TODO: More convenience functions for specific handle types, so that `check` closure doesn't have to be provided. Probably traits `Validate`, `Handle` and `Null` for that. Then something like `from_valid_handle_or_...()` (`E_HANDLE`) and `from_valid_or_...()` (`E_FAIL`). See also <https://github.com/microsoft/windows-rs/issues/2736>.
pub trait ResultExt<T> {
    /// Passes a non-zero `T` through to an `Ok` value, or, in case of it being zero, returns `Err` with [`windows::core::Error::from_win32()`].
    fn from_nonzero_or_win32(t: T) -> windows::core::Result<T>
    where
        T: num_traits::Zero;

    /// Passes a non-zero `T` through to an `Ok` value, or, in case of it being zero, returns `Err` with `HRESULT` `E_FAIL`.
    ///
    /// To be used with functions that don't offer an error code via `GetLastError()`.
    fn from_nonzero_or_e_fail(t: T) -> windows::core::Result<T>
    where
        T: num_traits::Zero;

    /// Passes a `T` through to an `Ok` value, if the check is successful, or otherwise returns `Err` with [`windows::core::Error::from_win32()`].
    fn from_checked_or_win32<F>(t: T, check: F) -> windows::core::Result<T>
    where
        F: FnOnce(&T) -> bool;

    /// Passes a `T` through to an `Ok` value, if the check is successful, or otherwise returns `Err` with `HRESULT` `E_FAIL`.
    ///
    /// To be used with functions that don't offer an error code via `GetLastError()`.
    fn from_checked_or_e_fail<F>(t: T, check: F) -> windows::core::Result<T>
    where
        F: FnOnce(&T) -> bool;
}

impl<T> ResultExt<T> for windows::core::Result<T> {
    fn from_nonzero_or_win32(t: T) -> windows::core::Result<T>
    where
        T: num_traits::Zero,
    {
        if t.is_zero() {
            Err(windows::core::Error::from_win32())
        } else {
            Ok(t)
        }
    }

    fn from_nonzero_or_e_fail(t: T) -> windows::core::Result<T>
    where
        T: num_traits::Zero,
    {
        if t.is_zero() {
            Err(E_FAIL.into())
        } else {
            Ok(t)
        }
    }

    fn from_checked_or_win32<F>(t: T, check: F) -> windows::core::Result<T>
    where
        F: FnOnce(&T) -> bool,
    {
        if check(&t) {
            Ok(t)
        } else {
            Err(windows::core::Error::from_win32())
        }
    }

    fn from_checked_or_e_fail<F>(t: T, check: F) -> windows::core::Result<T>
    where
        F: FnOnce(&T) -> bool,
    {
        if check(&t) {
            Ok(t)
        } else {
            Err(E_FAIL.into())
        }
    }
}

pub trait HResultExt {
    /// Like `ok()`, but with success `HRESULT`s forwarded instead of giving `()`. Useful when working with functions that can return multiple success return values, like `AssocQueryStringW()`.
    fn ok_with_hresult(self) -> windows::core::Result<HRESULT>;
}

impl HResultExt for HRESULT {
    fn ok_with_hresult(self) -> windows::core::Result<HRESULT> {
        if self.is_ok() {
            Ok(self)
        } else {
            Err(self.into())
        }
    }
}

#[cfg(all(test, feature = "windows_latest_compatible_all"))]
mod tests {
    use crate::windows::Win32::{
        Foundation::{ERROR_INSUFFICIENT_BUFFER, E_FAIL, E_UNEXPECTED, S_FALSE, S_OK},
        Globalization::{
            GetLocaleInfoEx, LOCALE_ICURRDIGITS, LOCALE_NAME_INVARIANT, LOCALE_RETURN_NUMBER,
        },
    };

    use crate::error::{HResultExt, ResultExt};

    #[test]
    fn result_ext_from_nonzero_or_win32() {
        let mut two_wide_chars = [u16::MAX, u16::MAX];
        assert_eq!(
            Result::from_nonzero_or_win32(unsafe {
                GetLocaleInfoEx(
                    LOCALE_NAME_INVARIANT,
                    LOCALE_ICURRDIGITS | LOCALE_RETURN_NUMBER,
                    Some(&mut two_wide_chars),
                )
            }),
            Ok(2)
        );
        let value = unsafe { *two_wide_chars.as_ptr().cast::<u32>() };
        assert!(value <= 2); // As per docs on `LOCALE_ICURRDIGITS`.

        assert_eq!(
            Result::from_nonzero_or_win32(unsafe {
                GetLocaleInfoEx(
                    LOCALE_NAME_INVARIANT,
                    LOCALE_ICURRDIGITS | LOCALE_RETURN_NUMBER,
                    Some(&mut [0]), // Invalid.
                )
            }),
            Err(ERROR_INSUFFICIENT_BUFFER.into())
        );
    }

    #[test]
    fn hresult_ext_ok_with_hresult() {
        assert_eq!(S_OK.ok_with_hresult(), Ok(S_OK));
        assert_eq!(S_FALSE.ok_with_hresult(), Ok(S_FALSE));
        assert_eq!(E_FAIL.ok_with_hresult(), Err(E_FAIL.into()));
        assert_eq!(E_UNEXPECTED.ok_with_hresult(), Err(E_UNEXPECTED.into()));
    }
}
