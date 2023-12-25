use crate::{windows, Null, ValidateHandle};
use windows::{
    core::HRESULT,
    Win32::Foundation::{E_FAIL, E_HANDLE},
};

pub trait ResultExt<T> {
    /// Returns `Ok(())`, or `Err`, based on [`windows::core::Error::from_win32()`].
    fn from_win32() -> windows::core::Result<()>;

    /// Returns `Err` with [`windows::core::Error::from_win32()`].
    fn err_from_win32() -> windows::core::Result<T>;

    /// Passes a non-zero `T` through to an `Ok` value, or, in case of it being zero, returns `Err` with [`windows::core::Error::from_win32()`], if it yields an error code, or zero otherwise.
    ///
    /// To be used with functions that communicate valid values as well as errors by a return value of 0 and need an additional call to `GetLastError()` to check whether an error has occurred (e.g., `SetWindowLongPtrW()`). Note that, depending on whether the function calls `SetLastError()` under all circumstances, you may need to precede the call for which you call this function by `SetLastError(ERROR_SUCCESS)`.
    fn from_nonzero_and_win32(t: T) -> windows::core::Result<T>
    where
        T: num_traits::Zero;

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

    /// Passes a non-null `T` through to an `Ok` value, or, in case of it being null, returns `Err` with `HRESULT` `E_HANDLE`.
    ///
    /// To be used with functions like `CreateBitmap()` that return a handle type or null, not offering an error code via `GetLastError()`.
    fn from_nonnull_or_e_handle(t: T) -> windows::core::Result<T>
    where
        T: Null;

    /// Passes a `T`, if successfully validated with `is_invalid()`, through to an `Ok` value, or, in case of it being invalid, returns `Err` with `HRESULT` `E_HANDLE`.
    ///
    /// To be used with functions that don't offer an error code via `GetLastError()`, and when there's a need to validate with `is_invalid()`.
    fn from_valid_or_e_handle(t: T) -> windows::core::Result<T>
    where
        T: ValidateHandle;

    /// Passes a `T` through to an `Ok` value, if the check is successful, or otherwise returns `Err` with [`windows::core::Error::from_win32()`].
    ///
    /// Can be used transitionally in new cases, until this crate might offer a more suitable solution.
    fn from_checked_or_win32<F>(t: T, check: F) -> windows::core::Result<T>
    where
        F: FnOnce(&T) -> bool;

    /// Passes a `T` through to an `Ok` value, if the check is successful, or otherwise returns `Err` with `HRESULT` `E_FAIL`.
    ///
    /// To be used with functions that don't offer an error code via `GetLastError()`.
    ///
    /// Can be used transitionally in new cases, until this crate might offer a more suitable solution.
    fn from_checked_or_e_fail<F>(t: T, check: F) -> windows::core::Result<T>
    where
        F: FnOnce(&T) -> bool;
}

impl<T> ResultExt<T> for windows::core::Result<T> {
    fn from_win32() -> windows::core::Result<()> {
        let error = windows::core::Error::from_win32();
        if error.code().is_ok() {
            Ok(())
        } else {
            Err(error)
        }
    }

    fn err_from_win32() -> windows::core::Result<T> {
        Err(windows::core::Error::from_win32())
    }

    fn from_nonzero_and_win32(t: T) -> windows::core::Result<T>
    where
        T: num_traits::Zero,
    {
        if t.is_zero() {
            let error = windows::core::Error::from_win32();
            if error.code().is_ok() {
                Ok(t)
            } else {
                Err(error)
            }
        } else {
            Ok(t)
        }
    }

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

    fn from_nonnull_or_e_handle(t: T) -> windows::core::Result<T>
    where
        T: Null,
    {
        if t.is_null() {
            Err(E_HANDLE.into())
        } else {
            Ok(t)
        }
    }

    fn from_valid_or_e_handle(t: T) -> windows::core::Result<T>
    where
        T: ValidateHandle,
    {
        if t.is_invalid() {
            Err(E_HANDLE.into())
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
    use crate::{
        core::{HResultExt, ResultExt},
        windows,
    };
    use windows::Win32::{
        Foundation::{ERROR_INSUFFICIENT_BUFFER, E_FAIL, E_UNEXPECTED, S_FALSE, S_OK},
        Globalization::{
            GetLocaleInfoEx, LOCALE_ICURRDIGITS, LOCALE_NAME_INVARIANT, LOCALE_RETURN_NUMBER,
        },
    };

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
